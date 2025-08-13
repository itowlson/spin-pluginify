fn main() {
    let git2 = vergen_git2::Git2Builder::default()
        .commit_date(true)
        .sha(true)
        .build()
        .expect("failed to build git information");
    vergen_git2::Emitter::default()
        .add_instructions(&git2)
        .expect("failed to add git instructions")
        .emit()
        .expect("failed to extract build information");
}
