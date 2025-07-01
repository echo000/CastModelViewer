use embed_manifest::{
    embed_manifest,
    manifest::{HeapType, Setting},
    new_manifest,
};
#[cfg(target_os = "windows")]
fn main() {
        embed_manifest(
            new_manifest("echo000.Cast")
                .heap_type(HeapType::SegmentHeap)
                .long_path_aware(Setting::Enabled),
        )
        .expect("unable to embed manifest file");
    println!("cargo:rerun-if-changed=build.rs");
}
#[cfg(not(target_os = "windows"))]
fn main() {}
