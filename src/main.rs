use std::io::Write;

use bip32;
use dialoguer;

const SELF_VERSION: &str = env!("CARGO_PKG_VERSION");
const BEFORE_EXIT_SLEEP: std::time::Duration = std::time::Duration::from_secs(60);
const XPUBS_PATH: &str = "xpubs.csv";
const XPUBS_COUNT: usize = 12000;

type ExitOr<T = ()> = Result<T, Box<dyn std::error::Error>>;


fn main() -> ExitOr {
    println!("[ 4th depth xpubs generator ]");
    println!("Version: {}", SELF_VERSION);
    println!("RECOMMEND: disable your network connection while copy/pasting your mnemonic phrase");

    let mnemonic_words = dialoguer::Input
        ::new()
        .with_prompt("Enter 24 mnemonic words")
        .interact_text()
        .map_err(|err| show_error(Box::new(err), "Cannot read from stdin"))?
    ;
    gen_xpubs(mnemonic_words)?;

    println!("\n\nDone. Check out {} file", XPUBS_PATH);
    std::thread::sleep(BEFORE_EXIT_SLEEP);
    Ok(())
}

fn gen_xpubs(mnemonic_words: String) -> ExitOr {
    let mnemonic = bip32::Mnemonic
        ::new(mnemonic_words, bip32::Language::English)
        .map_err(|err| show_error(Box::new(err), "Invalid mnemonic phrase, ensure it's correct (check either length should be = 24)"))?
    ;
    let seed = &mnemonic.to_seed("");

    let xpubs_path = std::path::Path::new(XPUBS_PATH);
    if xpubs_path.exists() {
        std::fs::remove_file(xpubs_path).map_err(
            |err| show_error(Box::new(err), "Cannot renew xpubs file")
        )?;
    }
    let mut xpubs_file = std::fs::File::create(xpubs_path).map_err(
        |err| show_error(Box::new(err), "Cannot create xpubs file")
    )?;
    xpubs_file.write_all(b"coin_type,xpub")
        .map_err(|err: _| show_error(Box::new(err), "Cannot write into the xpubs file"))?;

    for coin_type in kdam::prelude::tqdm!(
        0..XPUBS_COUNT,
        desc = "Generating XPubs 4th depth   ",
        animation = "arrow",
        position = 1
    ) {
        let child_path = format!("m/44'/{}'/0'/0", coin_type);
        let child_xprv = bip32::XPrv::derive_from_path(&seed, &child_path.parse()?)?;
        let child_xpub = child_xprv.public_key().to_string(bip32::Prefix::XPUB);

        let mut new_file_line = String::new();
        new_file_line.push('\n');
        new_file_line.push_str(&coin_type.to_string());
        new_file_line.push(',');
        new_file_line.push_str(&child_xpub);

        xpubs_file.write_all(new_file_line.as_bytes())
            .map_err(|err: _| show_error(Box::new(err), "Cannot write into the xpubs file"))?;
    }

    Ok(())
}


fn show_error(err: Box<dyn std::error::Error>, text: &'static str) -> Box<dyn std::error::Error> {
    eprintln!("ERROR: {} ({})", text, err.to_string());
    std::thread::sleep(BEFORE_EXIT_SLEEP);
    err
}
