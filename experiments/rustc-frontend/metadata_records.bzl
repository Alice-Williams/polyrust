"""One closed, control-safe graph serializer for metadata and source checking."""

def _add_record(arguments, fields):
    # Multiline params preserve arguments only when fields cannot inject records.
    # Validate inherited provider fields here too; Rust checks the full grammar.
    for value in fields:
        if any([character < " " or character == "\177" for character in value.elems()]):
            fail("Rust metadata record contains a control character")
    arguments.add_all(fields)

def crate_graph_arguments(ctx, root_key, crates):
    """Serialize immutable provider records into one bounded-parser response file.

    Args:
        ctx: The rule action context.
        root_key: The declared root defining key.
        crates: The closed defining-key map from RustSourceCrateInfo.

    Returns:
        An Args object with always-enabled multiline parameter-file transport.
    """
    arguments = ctx.actions.args()
    arguments.use_param_file("@%s", use_always = True)
    arguments.set_param_file_format("multiline")
    _add_record(arguments, ["--root", root_key])
    for key in sorted(crates):
        record = crates[key]
        _add_record(arguments, ["--crate", record.crate_name, key, record.source.path, record.source_name, record.metadata.path])
        for item in record.inputs:
            _add_record(arguments, ["--input", item.file.path, item.logical_name])
        for dependency in record.dependencies:
            _add_record(arguments, ["--dependency", dependency.alias, dependency.key])
    return arguments
