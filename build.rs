extern crate cpp_build;
extern crate gcc;

fn main() {
//    println!("cargo:rustc-link-lib=static:/Users/clay/repos/sliced/src/libredis.a");
//    println!("cargo:rustc-link-lib=static:/Users/clay/redis/redis/deps/jemalloc/lib/libjemalloc.a");
    // Build a Redis pseudo-library so that we have symbols that we can link
    // against while building Rust code.
    //
    // include/redismodule.h is just vendored in from the Redis project and
    // src/redismodule.c is just a stub that includes it and plays a few other
    // tricks that we need to complete the build.
    gcc::Build::new()
        .file("src/redismodule.c")
        .file("src/c/endianconv.c")
        .file("src/c/zmalloc.c")
        .file("src/c/listpack.c")
        .file("src/c/rax.c")
        .file("src/c/sds.c")
        .file("src/c/siphash.c")
        .file("src/c/sha1.c")
        .file("src/c/dict.c")
        .include("src/c/")
        .compile("libredismodule.a");

//    gcc::compile_library()

//    gcc::Build::new()
//        .file("src/listpack.c")
//        .include("include/")
//        .compile("liblistpack.a");
    // The GCC module emits `rustc-link-lib=static=redismodule` for us.

    cpp_build::build("src/lib.rs");
}
