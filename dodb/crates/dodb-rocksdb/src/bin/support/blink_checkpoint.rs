use dodb_server::{BlinkCheckpointExecution, BlinkWalLayout};

#[derive(Clone, Copy, Debug, Default)]
pub struct Options {
    pub layout: BlinkWalLayout,
    pub execution: BlinkCheckpointExecution,
}

impl Options {
    pub fn parse(
        layout: Option<&str>,
        execution: Option<&str>,
        segment_bytes: Option<&str>,
        blink_backend: bool,
    ) -> Result<Self, String> {
        if !blink_backend && (layout.is_some() || execution.is_some() || segment_bytes.is_some()) {
            return Err(
                "B-link checkpoint execution/layout options require --backend blink".to_owned(),
            );
        }
        let execution = match execution.unwrap_or("sync") {
            "sync" => BlinkCheckpointExecution::Sync,
            "background" => BlinkCheckpointExecution::Background,
            value => return Err(format!("unsupported checkpoint execution {value}")),
        };
        let layout = match layout.unwrap_or("legacy") {
            "legacy" if segment_bytes.is_none() => BlinkWalLayout::Legacy,
            "segmented" => {
                let bytes = segment_bytes
                    .unwrap_or("33554432")
                    .parse::<u64>()
                    .map_err(|_| "invalid --blink-wal-segment-bytes".to_owned())?;
                if bytes == 0 {
                    return Err("--blink-wal-segment-bytes must be positive".to_owned());
                }
                BlinkWalLayout::Segmented {
                    max_segment_bytes: bytes,
                }
            }
            value => {
                return Err(format!(
                    "invalid WAL layout or segment-size combination {value}"
                ));
            }
        };
        if execution == BlinkCheckpointExecution::Background && layout == BlinkWalLayout::Legacy {
            return Err("background checkpoints require --blink-wal-layout segmented".to_owned());
        }
        Ok(Self { layout, execution })
    }

    pub fn configuration(self) -> serde_json::Value {
        serde_json::json!({
            "layout": match self.layout { BlinkWalLayout::Legacy => "legacy", BlinkWalLayout::Segmented { .. } => "segmented" },
            "segment_bytes": match self.layout { BlinkWalLayout::Legacy => None, BlinkWalLayout::Segmented { max_segment_bytes } => Some(max_segment_bytes) },
            "execution": match self.execution { BlinkCheckpointExecution::Sync => "sync", BlinkCheckpointExecution::Background => "background" },
            "retained_wal_limit_bytes": 512 * 1024 * 1024_u64,
            "retained_image_limit_bytes": 512 * 1024 * 1024_u64,
        })
    }
}
