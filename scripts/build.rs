
mod shift_jis;

fn main() {
    shift_jis::generate().expect("shift-jis table generation failed");

    println!("cargo:rerun-if-changed=scripts/shift_jis.rs");
    println!("cargo:rerun-if-changed=scripts/build.rs");
}
