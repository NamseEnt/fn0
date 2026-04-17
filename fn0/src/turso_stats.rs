use bytes::Bytes;
use opentelemetry::KeyValue;
use opentelemetry::metrics::Counter;
use std::io::Read;
use std::sync::OnceLock;
use struson::reader::{JsonReader, JsonStreamReader, ValueType};
use tokio::runtime::Handle;
use tokio::sync::mpsc::UnboundedReceiver;

pub struct ParseJob {
    pub code_id: String,
    pub bytes_rx: UnboundedReceiver<Bytes>,
}

static PARSER: OnceLock<crossbeam_channel::Sender<ParseJob>> = OnceLock::new();

pub fn submit(job: ParseJob) {
    let tx = PARSER.get_or_init(|| {
        let handle = Handle::current();
        let (tx, rx) = crossbeam_channel::unbounded::<ParseJob>();
        std::thread::Builder::new()
            .name("turso-parse".into())
            .spawn(move || {
                let _guard = handle.enter();
                parser_loop(rx);
            })
            .expect("failed to spawn turso-parse thread");
        tx
    });
    if let Err(err) = tx.send(job) {
        tracing::warn!(%err, "turso parse thread queue closed");
    }
}

fn parser_loop(rx: crossbeam_channel::Receiver<ParseJob>) {
    while let Ok(job) = rx.recv() {
        let code_id = job.code_id.clone();
        let reader = BlockingBytesReader::new(job.bytes_rx);
        match parse_pipeline_response(reader) {
            Ok(totals) => emit_metrics(&code_id, totals),
            Err(err) => tracing::debug!(%err, %code_id, "turso response parse error"),
        }
    }
}

fn emit_metrics(code_id: &str, totals: Totals) {
    let subdomain = code_id.split("::").next().unwrap_or(code_id).to_string();
    let attrs = [KeyValue::new("code_id", subdomain)];
    if totals.rows_read > 0 {
        rows_read_counter().add(totals.rows_read, &attrs);
    }
    if totals.rows_written > 0 {
        rows_written_counter().add(totals.rows_written, &attrs);
    }
}

fn rows_read_counter() -> &'static Counter<u64> {
    static COUNTER: OnceLock<Counter<u64>> = OnceLock::new();
    COUNTER.get_or_init(|| {
        opentelemetry::global::meter("fn0-worker")
            .u64_counter("turso.rows_read")
            .with_description("Rows read reported by Turso hrana responses")
            .build()
    })
}

fn rows_written_counter() -> &'static Counter<u64> {
    static COUNTER: OnceLock<Counter<u64>> = OnceLock::new();
    COUNTER.get_or_init(|| {
        opentelemetry::global::meter("fn0-worker")
            .u64_counter("turso.rows_written")
            .with_description("Rows written reported by Turso hrana responses")
            .build()
    })
}

#[derive(Default)]
struct Totals {
    rows_read: u64,
    rows_written: u64,
}

fn parse_pipeline_response<R: Read>(reader: R) -> Result<Totals, struson::reader::ReaderError> {
    let mut json = JsonStreamReader::new(reader);
    let mut totals = Totals::default();

    json.begin_object()?;
    while json.has_next()? {
        let key = json.next_name_owned()?;
        if key == "results" {
            json.begin_array()?;
            while json.has_next()? {
                walk_stream_result(&mut json, &mut totals)?;
            }
            json.end_array()?;
        } else {
            json.skip_value()?;
        }
    }
    json.end_object()?;
    Ok(totals)
}

fn walk_stream_result<R: Read>(
    json: &mut JsonStreamReader<R>,
    totals: &mut Totals,
) -> Result<(), struson::reader::ReaderError> {
    if json.peek()? != ValueType::Object {
        json.skip_value()?;
        return Ok(());
    }
    json.begin_object()?;
    while json.has_next()? {
        let key = json.next_name_owned()?;
        if key == "response" {
            walk_response(json, totals)?;
        } else {
            json.skip_value()?;
        }
    }
    json.end_object()?;
    Ok(())
}

fn walk_response<R: Read>(
    json: &mut JsonStreamReader<R>,
    totals: &mut Totals,
) -> Result<(), struson::reader::ReaderError> {
    if json.peek()? != ValueType::Object {
        json.skip_value()?;
        return Ok(());
    }
    json.begin_object()?;
    while json.has_next()? {
        let key = json.next_name_owned()?;
        if key == "result" {
            walk_result_payload(json, totals)?;
        } else {
            json.skip_value()?;
        }
    }
    json.end_object()?;
    Ok(())
}

fn walk_result_payload<R: Read>(
    json: &mut JsonStreamReader<R>,
    totals: &mut Totals,
) -> Result<(), struson::reader::ReaderError> {
    if json.peek()? != ValueType::Object {
        json.skip_value()?;
        return Ok(());
    }
    json.begin_object()?;
    while json.has_next()? {
        let key = json.next_name_owned()?;
        match key.as_str() {
            "rows_read" => {
                if let Ok(Ok(n)) = json.next_number::<u64>() {
                    totals.rows_read += n;
                }
            }
            "rows_written" => {
                if let Ok(Ok(n)) = json.next_number::<u64>() {
                    totals.rows_written += n;
                }
            }
            "step_results" => {
                json.begin_array()?;
                while json.has_next()? {
                    walk_result_payload(json, totals)?;
                }
                json.end_array()?;
            }
            _ => json.skip_value()?,
        }
    }
    json.end_object()?;
    Ok(())
}

struct BlockingBytesReader {
    rx: UnboundedReceiver<Bytes>,
    leftover: Option<Bytes>,
}

impl BlockingBytesReader {
    fn new(rx: UnboundedReceiver<Bytes>) -> Self {
        Self {
            rx,
            leftover: None,
        }
    }
}

impl Read for BlockingBytesReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        loop {
            if let Some(mut chunk) = self.leftover.take() {
                let n = chunk.len().min(buf.len());
                buf[..n].copy_from_slice(&chunk[..n]);
                if n < chunk.len() {
                    self.leftover = Some(chunk.split_off(n));
                }
                return Ok(n);
            }
            match self.rx.blocking_recv() {
                Some(chunk) if chunk.is_empty() => continue,
                Some(chunk) => self.leftover = Some(chunk),
                None => return Ok(0),
            }
        }
    }
}
