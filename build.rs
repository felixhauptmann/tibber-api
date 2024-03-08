fn main() {
    cynic_codegen::register_schema("tibber")
        .from_sdl_file("graphql/tibber.graphql")
        .unwrap()
        .as_default()
        .unwrap();
}
