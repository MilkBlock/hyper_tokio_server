import asyncio
import _thread
import time
import websockets


# async def hello():
#     uri = "ws://1.13.2.149:11451"
#     async with websockets.connect(uri) as websocket:
#         message = "register_referee:(0)"
#         await websocket.send(message)
#         print(f"Sent:{message}")
#         response = await websocket.recv()
#         print(f"Receive:{response}")


# asyncio.get_event_loop().run_until_complete(hello())


# async def request_set_board_coords(web_url, coordinate):
#     async with websockets.connect(websocket_url) as websocket:
#         for x, y, track_id in coordinate:
#             message = f"request_set_board_coordinate:({track_id},{x},{y})"
#             await websocket.send(message)
#             print(f"Sent: {message}")
#             response = await websocket.recv()
#             print(f"Received: {response}")

websocket_url = "ws://1.13.2.149:11451/"
# coordinates = [((x1 + x2) / 2, (y1 + y2) / 2, track_id) for x1, y1, x2, y2, _, _, track_id in pred_boxes] #pred_boxes中包含了坐标和track_id信息
# asyncio.get_event_loop().run_until_complete(request_set_board_coords(websocket_url, coordinates))

async def init_as_referee(room_num):
    referee_ws = websockets.connect("ws://1.13.2.149:11451")
    referee_ws.run_forever();
    await referee_ws.send(f"register_referee({room_num})")
    return referee_ws
    
async def init_as_board(wifi="Met"):
    board_ws = await websockets.connect(websocket_url)
    board_ws.run_forever();
    await board_ws.send(f"regsiter_board(\"{wifi}\",\"192.168.43.114\")")
    return board_ws

async def init_as_app(board_idx,name,room_num):
    app_ws = await websockets.connect(websocket_url)
    app_ws.send(f"regsiter_app(\"{board_idx}\",{board_idx},{room_num})")
    return app_ws

async def request_list_rooms(app_ws:websockets.WebSocketClientProtocol):
    await app_ws.send(f"request_list_rooms()")
    print(await app_ws.recv())

async def request_list_boards_in_room(app_ws:websockets.WebSocketClientProtocol,room_num):
    await app_ws.send(f"request_list_boards_in_room({room_num})",)
    print(await app_ws.recv())

    
async def  request_set_board_coords(board_ws:websockets.WebSocketClientProtocol,board_idx,x,y):
    await board_ws.send(f"request_set_board_coordinate:({board_idx},{x},{y})")

    
