pub fn init_tracing() {
    let subscriber = tracing_subscriber::fmt::Subscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .with_timer(tracing_subscriber::fmt::time::ChronoLocal::new(
            "[%H:%M:%S%.3f]".to_string(),
        ))
        .with_target(false)
        .with_thread_ids(true)
        .with_thread_names(true)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();
}
