//! Sous Windows, incruste l'icone (la tete d'Acaboulette) dans l'executable.
//! Ailleurs, ne fait rien.

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=data/icons/packarch-acaboulette.ico");

    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    if env::var("CARGO_CFG_TARGET_ENV").as_deref() != Ok("gnu") {
        println!("cargo:warning=icone de l'exe non incrustee (seule la cible windows-gnu est geree)");
        return;
    }

    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let ico = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("data/icons/packarch-acaboulette.ico");
    let rc = out.join("icone.rc");
    let obj = out.join("icone.o");
    // Les antislashs sont des echappements dans un .rc : on met des slashs.
    let ico = ico.to_string_lossy().replace('\\', "/");
    std::fs::write(&rc, format!("1 ICON \"{ico}\"\n")).unwrap();

    // Compilation croisee : windres prefixe ; sous Windows : windres tout court.
    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let candidates = [format!("{arch}-w64-mingw32-windres"), "windres".to_owned()];
    let ok = candidates.iter().any(|windres| {
        Command::new(windres)
            .arg("--output-format=coff")
            .arg(&rc)
            .arg(&obj)
            .status()
            .is_ok_and(|s| s.success())
    });
    if ok {
        println!("cargo:rustc-link-arg-bins={}", obj.display());
    } else {
        println!("cargo:warning=windres introuvable : l'exe n'aura pas d'icone");
    }
}
