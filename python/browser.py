''' Browser window '''
import typing as t
import tkinter
import ewb

if t.TYPE_CHECKING:
    from ewb import PyNode
    from tkinter import Event, Misc

WIDTH = 800
HEIGHT = 600
HSTEP, VSTEP = 13, 18
SCROLL_STEP = 100

XYChar = t.Tuple[float, float, str]

class Browser:
    '''
    Browser class
    '''
    def __init__(self) -> None:
        self.window = tkinter.Tk()
        self.canvas = tkinter.Canvas(self.window, width=WIDTH, height=HEIGHT)
        self.canvas.pack()
        self.scroll = 0
        self.window.bind("<Down>", func=self.scrolldown)

    def load(self, url: str):
        response = ewb.request(url)
        node = ewb.load(response)
        text_nodes = node.get_text_nodes()

        self.display_list = Browser.layout(text_nodes)
        self.draw()

    @staticmethod
    def layout(text_nodes: t.List['PyNode']) -> t.List[XYChar]:
        display_list: t.List[XYChar] = []
        cursor_x, cursor_y = HSTEP, VSTEP

        for node in text_nodes:
            text = node.data.attributes.get('content')
            if text:
                for char in text:
                    display_list.append((cursor_x, cursor_y, char))
                    cursor_x += HSTEP
                    if cursor_x >= WIDTH - HSTEP:
                        cursor_y += VSTEP
                        cursor_x = HSTEP

        return display_list

    def draw(self):
        self.canvas.delete('all')
        for x, y, c in self.display_list:
            if y > self.scroll + HEIGHT: continue
            if y + VSTEP < self.scroll: continue
            self.canvas.create_text(x, y - self.scroll, text=c)

    def scrolldown(self, event: "Event[Misc]"):
        self.scroll += SCROLL_STEP
        self.draw()
