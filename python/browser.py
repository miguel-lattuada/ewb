''' Browser window '''
import tkinter

WIDTH = 800
HEIGHT = 600

class Browser:
    '''
    Browser class
    '''
    def __init__(self) -> None:
        self.window = tkinter.Tk()
        self.canvas = tkinter.Canvas(self.window, width=WIDTH, height=HEIGHT)
        self.canvas.pack()

    def load(self, url: str):
        ''' Draw canvas '''
        self.canvas.create_rectangle(10, 20, 400, 300)
        self.canvas.create_oval(100, 100, 150, 150)
        self.canvas.create_text(200, 150, text="Hi!")
