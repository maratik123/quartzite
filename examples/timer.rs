use quartzite::prelude::*;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use std::time::Duration;

fn main() {
    env_logger::init();
    let app = Application::new().expect("only one Application per process");

    let counter = Arc::new(AtomicUsize::new(0));
    let counter2 = Arc::clone(&counter);

    let mut timer = Timer::new(Duration::from_millis(50));
    timer.connect_timeout(move |_| {
        let n = counter2.fetch_add(1, Ordering::SeqCst) + 1;
        println!("tick {n}");
        if n >= 3 {
            Application::global()
                .expect("Application must exist")
                .quit();
        }
    });
    timer.start(app.event_loop().sender());

    app.exec();
    println!("done after {} ticks", counter.load(Ordering::SeqCst));
}
