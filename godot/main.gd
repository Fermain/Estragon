extends Control
## Loads a fixture document and displays it. Change `document` to try others.

@export_file("*.ui.json") var document := "res://fixtures/hud.ui.json"

func _ready() -> void:
	var ui := Estragon.load_document(document)
	if ui:
		add_child(ui)
		ui.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
