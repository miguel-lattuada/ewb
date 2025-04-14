''' Browser window '''
import typing as t
import tkinter

import minify_html

import ewb

WIDTH = 800
HEIGHT = 600
HSTEP, VSTEP = 13, 18

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
        response = t.cast(str, ewb.request(url))

        response = minify_html.minify(
            code=response,
            minify_css=True,
            minify_js=True,
            keep_closing_tags=True,
            preserve_brace_template_syntax=True,
            keep_html_and_head_opening_tags=True,
        ).replace('<!doctype html>', '')

        print(response)

        node = ewb.load(response)
        text_nodes = node.get_text_nodes()

        print(len(text_nodes))

        cursor_x, cursor_y = HSTEP, VSTEP
        for text_node in text_nodes:
            text = text_node['data']['attributes']['content']

            for char in text:
                self.canvas.create_text(
                    cursor_x, cursor_y, text=char)
                cursor_x += HSTEP

                if cursor_x >= WIDTH - HSTEP:
                    cursor_y += VSTEP
                    cursor_x = HSTEP
