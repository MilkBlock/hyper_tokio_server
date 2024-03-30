from ws import *
async def main():
    board_ws = await init_as_board("Metro")
    await request_list_rooms(board_ws)

