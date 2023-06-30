use klask::Settings;
use watchdog_demo::{run, Arguments};

fn main() {
    klask::run_derived::<Arguments, _>(Settings::default(), |args| {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        if let Err(e) = rt.block_on(async { run::<true>(&args).await }) {
            eprintln!("{e:?}");
        }
    });
}
