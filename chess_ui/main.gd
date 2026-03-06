extends Node2D

# TODO: Typing

const SPRITE_ORDER = ["K", "Q", "R", "B", "N", "P"]
const SPRITES = [preload("res://assets/wK.png"), preload("res://assets/wQ.png"),
				 preload("res://assets/wR.png"), preload("res://assets/wB.png"),
				 preload("res://assets/wN.png"), preload("res://assets/wP.png"),
				 preload("res://assets/bK.png"), preload("res://assets/bQ.png"),
				 preload("res://assets/bR.png"), preload("res://assets/bB.png"),
				 preload("res://assets/bN.png"), preload("res://assets/bP.png")]

var engine_interface = EngineInterface.new_interface("", 3, false)
@onready var engine_thread = Thread.new()
@onready var mutex = Mutex.new()
var players_move = true
var selected_square = null
@onready var thinking_label = $GameUI/Thinking

func _ready() -> void:
	display_board(engine_interface.get_board())

func display_board(board):
	# Reset previous board
	for child in get_children():
		if child is Sprite2D and child.name != "Board":
			child.queue_free()

	for i in range(64):
		if board[i] == "":
			continue

		var sprite_index = SPRITE_ORDER.find(board[i].to_upper())
		if board[i] == board[i].to_lower():
			sprite_index += 6

		var x = i % 8 * 100 + 50
		var y = (7 - i / 8) * 100 + 50

		var sprite = Sprite2D.new()
		sprite.texture = SPRITES[sprite_index]
		sprite.position = Vector2(x, y)
		add_child(sprite)

func _input(event):
	if players_move and event is InputEventMouseButton \
		and event.button_index == MOUSE_BUTTON_LEFT and event.pressed:
		var click_position = (event.position / 100).floor()
		var i = int(click_position.x + (7 - click_position.y) * 8)
		
		handle_click(i)

func handle_click(i):
	if selected_square == null:
		if engine_interface.valid_square_selection(i):
			selected_square = i
			# TODO: Show possible moves
	else:
		if engine_interface.valid_move(selected_square, i):
			engine_interface.make_move()
			display_board(engine_interface.get_board())
			# thinking_label.visible = true
			players_move = false

			# engine_thread.wait_to_finish()
			engine_thread.start(engine_move)
			engine_thread.wait_to_finish()
		selected_square = null

func engine_move():
	# mutex.lock()
	# engine_interface.engine_move()
	# mutex.unlock()
	# pass
	# engine_interface.engine_move()
	# display_board(engine_interface.get_board())
	thinking_label.visible = false
	print("Invisible!")
	print(thinking_label.visible)
	players_move = true

func _exit_tree():
	engine_thread.wait_to_finish()
