fn main() {
    let link_flags = [
        "NODEFAULTLIB",
        "SAFESEH:NO",
        "ENTRY:orbs_main",
        "OPT:ICF=999",
        "OPT:REF",
        "MERGE:.rdata=.text",
        "MERGE:.pdata=.text",
        "EMITPOGOPHASEINFO",
        // "DEBUG:NONE",
    ];

    link_flags
        .iter()
        .for_each(|flag| println!("cargo:rustc-link-arg=/{}", flag));
}
