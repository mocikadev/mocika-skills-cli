fn main() {
    if let Err(err) = skm::run() {
        eprintln!(
            "{}: {err:#}",
            console::style(skm::i18n::t("error")).red().bold()
        );
        std::process::exit(1);
    }
}
