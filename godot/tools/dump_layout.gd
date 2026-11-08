extends SceneTree
## Headless parity dump: lays out a document in Godot and prints every node's rect as JSON.
## Usage: godot --headless --path godot -s res://tools/dump_layout.gd -- <res://doc.ui.json>

func _init() -> void:
	var args := OS.get_cmdline_user_args()
	var path: String = args[0] if args.size() > 0 else "res://fixtures/hud.ui.json"
	var ui := Estragon.load_document(path)
	if ui == null:
		quit(1)
		return
	var doc_size := ui.size
	root.add_child(ui)
	ui.position = Vector2.ZERO
	ui.size = doc_size
	# Containers sort on a deferred call; let a couple of frames pass.
	await process_frame
	await process_frame
	var out: Array = []
	_walk(ui, 0, out)
	print(JSON.stringify(out, "  "))
	quit(0)

func _walk(c: Control, depth: int, out: Array) -> void:
	var p := c.get_global_rect()
	out.append({"id": String(c.name), "depth": depth, "x": p.position.x, "y": p.position.y, "width": p.size.x, "height": p.size.y})
	for child in c.get_children():
		if child is Control:
			_walk(child, depth + 1, out)
