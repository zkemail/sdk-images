// Include the generated proto file
pub mod proto_blueprint {
    include!(concat!(env!("OUT_DIR"), "/blueprint.rs"));
}

// Note: We're not re-exporting the types at the module level
// to avoid conflicts. Access them via proto_types::proto_blueprint
