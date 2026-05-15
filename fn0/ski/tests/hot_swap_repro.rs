use bytes::Bytes;
use fn0_ski::{Request, Response, SkiInstance};
use http_body_util::{BodyExt, Empty};
use std::rc::Rc;
use tokio::task::LocalSet;

fn empty_request(url: &str) -> Request {
    let body = http_body_util::combinators::UnsyncBoxBody::new(
        Empty::<Bytes>::new().map_err(|never| match never {}),
    );
    hyper::Request::builder()
        .method("GET")
        .uri(url)
        .body(body)
        .unwrap()
}

const USER_JS: &str = "globalThis.handler = (req) => new Response('ok', { status: 200 });";

fn current_thread_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}

#[test]
fn a_load_then_drop_then_load() {
    let rt = current_thread_runtime();
    let local = LocalSet::new();
    local.block_on(&rt, async {
        for i in 0..5 {
            eprintln!("[repro] iteration {i}: SkiInstance::load");
            let instance = Rc::new(SkiInstance::load(USER_JS, "/entry.js", None).unwrap());

            let driver_inst = instance.clone();
            let driver = tokio::task::spawn_local(async move {
                driver_inst.drive_forever().await;
            });

            tokio::task::yield_now().await;
            eprintln!("[repro] iteration {i}: aborting driver + dropping instance");
            driver.abort();
            drop(instance);

            tokio::task::yield_now().await;
        }
        eprintln!("[repro] survived load/drop loop");
    });
}

#[test]
fn b_load_call_drop_load_call() {
    let rt = current_thread_runtime();
    let local = LocalSet::new();
    local.block_on(&rt, async {
        for i in 0..5 {
            eprintln!("[repro] iteration {i}: load + call + drop");
            let instance = Rc::new(SkiInstance::load(USER_JS, "/entry.js", None).unwrap());

            let driver_inst = instance.clone();
            let driver = tokio::task::spawn_local(async move {
                driver_inst.drive_forever().await;
            });

            let resp: Response = instance
                .call(empty_request("http://localhost/"))
                .await
                .unwrap();
            eprintln!("[repro] iteration {i}: response status {}", resp.status());
            let _ = resp.into_body().collect().await;

            driver.abort();
            drop(instance);

            tokio::task::yield_now().await;
        }
        eprintln!("[repro] survived load/call/drop loop");
    });
}

fn say(s: &str) {
    use std::io::Write;
    eprintln!("{s}");
    let _ = std::io::stderr().flush();
}

#[test]
fn c1_load_a_then_load_b_no_call_no_drop() {
    let rt = current_thread_runtime();
    let local = LocalSet::new();
    local.block_on(&rt, async {
        say("[c1] load A");
        let a = Rc::new(SkiInstance::load(USER_JS, "/a.js", None).unwrap());
        let a_driver_inst = a.clone();
        let a_driver = tokio::task::spawn_local(async move {
            a_driver_inst.drive_forever().await;
        });
        tokio::task::yield_now().await;

        say("[c1] load B while A alive (no call on either)");
        let b = Rc::new(SkiInstance::load(USER_JS, "/b.js", None).unwrap());

        say("[c1] cleanup");
        a_driver.abort();
        drop(a);
        drop(b);
        say("[c1] survived");
    });
}

#[test]
fn c2_load_a_call_a_then_load_b() {
    let rt = current_thread_runtime();
    let local = LocalSet::new();
    local.block_on(&rt, async {
        say("[c2] load A");
        let a = Rc::new(SkiInstance::load(USER_JS, "/a.js", None).unwrap());
        let a_driver_inst = a.clone();
        let a_driver = tokio::task::spawn_local(async move {
            a_driver_inst.drive_forever().await;
        });

        say("[c2] call A");
        let _ = a.call(empty_request("http://localhost/a")).await.unwrap();

        say("[c2] abort A driver (A still alive via local Rc)");
        a_driver.abort();

        say("[c2] load B while A alive");
        let b = Rc::new(SkiInstance::load(USER_JS, "/b.js", None).unwrap());

        say("[c2] cleanup");
        drop(a);
        drop(b);
        say("[c2] survived");
    });
}

#[test]
fn c4_drain_a_before_call_b() {
    let rt = current_thread_runtime();
    let local = LocalSet::new();
    local.block_on(&rt, async {
        say("[c4] load A + call A");
        let a = Rc::new(SkiInstance::load(USER_JS, "/a.js", None).unwrap());
        let a_driver_inst = a.clone();
        let a_driver = tokio::task::spawn_local(async move {
            a_driver_inst.drive_forever().await;
        });
        let _ = a.call(empty_request("http://localhost/a")).await.unwrap();

        say("[c4] load B + spawn B driver (A still alive)");
        let b = Rc::new(SkiInstance::load(USER_JS, "/b.js", None).unwrap());
        let b_driver_inst = b.clone();
        let b_driver = tokio::task::spawn_local(async move {
            b_driver_inst.drive_forever().await;
        });

        say("[c4] abort A driver, drop A, drain task queue thoroughly");
        a_driver.abort();
        drop(a);
        for _ in 0..50 {
            tokio::task::yield_now().await;
        }

        say("[c4] call B (A should be fully gone)");
        let _ = b.call(empty_request("http://localhost/b")).await.unwrap();

        b_driver.abort();
        drop(b);
        say("[c4] survived");
    });
}

#[test]
fn e_drop_order_lifo() {
    // Construct A then B, then drop B before A (LIFO order matches V8 enter stack).
    let rt = current_thread_runtime();
    let local = LocalSet::new();
    local.block_on(&rt, async {
        say("[e] load A");
        let a = Rc::new(SkiInstance::load(USER_JS, "/a.js", None).unwrap());
        say("[e] load B");
        let b = Rc::new(SkiInstance::load(USER_JS, "/b.js", None).unwrap());
        say("[e] drop B (LIFO)");
        drop(b);
        say("[e] drop A");
        drop(a);
        say("[e] survived LIFO drop");
    });
}

#[test]
fn f_drop_order_fifo() {
    let rt = current_thread_runtime();
    let local = LocalSet::new();
    local.block_on(&rt, async {
        say("[f] load A");
        let a = Rc::new(SkiInstance::load(USER_JS, "/a.js", None).unwrap());
        say("[f] load B");
        let b = Rc::new(SkiInstance::load(USER_JS, "/b.js", None).unwrap());
        say("[f] drop A (FIFO -- spec says any order is safe)");
        drop(a);
        say("[f] drop B");
        drop(b);
        say("[f] survived FIFO drop");
    });
}

#[test]
fn h_three_isolates_random_drop_order() {
    let rt = current_thread_runtime();
    let local = LocalSet::new();
    local.block_on(&rt, async {
        say("[h] load A, B, C");
        let a = Rc::new(SkiInstance::load(USER_JS, "/a.js", None).unwrap());
        let b = Rc::new(SkiInstance::load(USER_JS, "/b.js", None).unwrap());
        let c = Rc::new(SkiInstance::load(USER_JS, "/c.js", None).unwrap());
        say("[h] drop B (middle)");
        drop(b);
        say("[h] drop A (was bottom)");
        drop(a);
        say("[h] drop C");
        drop(c);
        say("[h] survived");
    });
}

#[test]
fn i_concurrent_drivers_two_calls_interleave() {
    let rt = current_thread_runtime();
    let local = LocalSet::new();
    local.block_on(&rt, async {
        say("[i] load A + spawn driver");
        let a = Rc::new(SkiInstance::load(USER_JS, "/a.js", None).unwrap());
        let a_driver = tokio::task::spawn_local({
            let a = a.clone();
            async move {
                a.drive_forever().await;
            }
        });

        say("[i] load B + spawn driver (both alive on same thread)");
        let b = Rc::new(SkiInstance::load(USER_JS, "/b.js", None).unwrap());
        let b_driver = tokio::task::spawn_local({
            let b = b.clone();
            async move {
                b.drive_forever().await;
            }
        });

        say("[i] tokio::join two calls — drivers must interleave isolate work");
        let (resp_a, resp_b) = tokio::join!(
            a.call(empty_request("http://localhost/a")),
            b.call(empty_request("http://localhost/b")),
        );
        assert_eq!(resp_a.unwrap().status(), 200);
        assert_eq!(resp_b.unwrap().status(), 200);

        a_driver.abort();
        b_driver.abort();
        drop(a);
        drop(b);
        say("[i] survived two-isolate concurrent calls");
    });
}

#[test]
fn j_many_isolates_call_interleave() {
    let rt = current_thread_runtime();
    let local = LocalSet::new();
    local.block_on(&rt, async {
        const N: usize = 8;
        say(&format!("[j] load {N} isolates with drivers"));
        let mut instances: Vec<Rc<SkiInstance>> = Vec::with_capacity(N);
        let mut drivers: Vec<tokio::task::JoinHandle<()>> = Vec::with_capacity(N);
        for i in 0..N {
            let inst = Rc::new(
                SkiInstance::load(USER_JS, &format!("/inst-{i}.js"), None).unwrap(),
            );
            let driver = tokio::task::spawn_local({
                let inst = inst.clone();
                async move {
                    inst.drive_forever().await;
                }
            });
            instances.push(inst);
            drivers.push(driver);
        }

        say("[j] join N calls");
        let calls = instances.iter().enumerate().map(|(i, inst)| {
            inst.call(empty_request(&format!("http://localhost/{i}")))
        });
        let results = futures::future::join_all(calls).await;
        for (i, r) in results.into_iter().enumerate() {
            assert_eq!(r.unwrap().status(), 200, "isolate {i}");
        }

        say("[j] drop drivers + instances in non-LIFO order");
        for d in drivers.drain(..) {
            d.abort();
        }
        // Drop in reverse of pop: front-first.
        for _ in 0..N {
            let _ = instances.remove(0);
        }
        say("[j] survived");
    });
}

#[test]
fn g_fifo_drop_then_v8_work_on_remaining() {
    let rt = current_thread_runtime();
    let local = LocalSet::new();
    local.block_on(&rt, async {
        say("[g] load A");
        let a = Rc::new(SkiInstance::load(USER_JS, "/a.js", None).unwrap());
        say("[g] load B");
        let b = Rc::new(SkiInstance::load(USER_JS, "/b.js", None).unwrap());
        let b_driver_inst = b.clone();
        let b_driver = tokio::task::spawn_local(async move {
            b_driver_inst.drive_forever().await;
        });

        say("[g] drop A (FIFO)");
        drop(a);

        say("[g] call B after FIFO drop of A");
        let _ = b.call(empty_request("http://localhost/b")).await.unwrap();

        b_driver.abort();
        drop(b);
        say("[g] survived FIFO drop + V8 work on B");
    });
}

#[test]
fn c3_full_coexist_call_both() {
    let rt = current_thread_runtime();
    let local = LocalSet::new();
    local.block_on(&rt, async {
        say("[c3] load A");
        let a = Rc::new(SkiInstance::load(USER_JS, "/a.js", None).unwrap());
        let a_driver_inst = a.clone();
        let a_driver = tokio::task::spawn_local(async move {
            a_driver_inst.drive_forever().await;
        });

        say("[c3] call A");
        let _ = a.call(empty_request("http://localhost/a")).await.unwrap();

        say("[c3] abort A driver");
        a_driver.abort();

        say("[c3] load B while A alive");
        let b = Rc::new(SkiInstance::load(USER_JS, "/b.js", None).unwrap());
        let b_driver_inst = b.clone();
        let b_driver = tokio::task::spawn_local(async move {
            b_driver_inst.drive_forever().await;
        });

        say("[c3] drop A");
        drop(a);

        say("[c3] call B");
        let _ = b.call(empty_request("http://localhost/b")).await.unwrap();

        say("[c3] cleanup");
        b_driver.abort();
        drop(b);
        say("[c3] survived");
    });
}

#[test]
fn d_load_drop_load_strict_serial() {
    let rt = current_thread_runtime();
    let local = LocalSet::new();
    local.block_on(&rt, async {
        eprintln!("[repro/d] step 1: load A");
        let a = Rc::new(SkiInstance::load(USER_JS, "/a.js", None).unwrap());
        let a_driver_inst = a.clone();
        let a_driver = tokio::task::spawn_local(async move {
            a_driver_inst.drive_forever().await;
        });

        eprintln!("[repro/d] step 2: call A");
        let _ = a
            .call(empty_request("http://localhost/a"))
            .await
            .unwrap();

        eprintln!("[repro/d] step 3: abort A driver, drop A, drain task queue");
        a_driver.abort();
        drop(a);
        for _ in 0..20 {
            tokio::task::yield_now().await;
        }

        eprintln!("[repro/d] step 4: load B (A and its driver task already dropped)");
        let b = Rc::new(SkiInstance::load(USER_JS, "/b.js", None).unwrap());
        let b_driver_inst = b.clone();
        let b_driver = tokio::task::spawn_local(async move {
            b_driver_inst.drive_forever().await;
        });

        eprintln!("[repro/d] step 5: call B");
        let _ = b
            .call(empty_request("http://localhost/b"))
            .await
            .unwrap();

        b_driver.abort();
        drop(b);

        eprintln!("[repro/d] survived strict serial");
    });
}
