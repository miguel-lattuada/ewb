import typing as t
import tkinter
import tkinter.font

import ewb

XYTextFont = t.Tuple[float, float, str, tkinter.font.Font]
FontWeight = t.Literal['normal', 'bold']
FontStyle = t.Literal['roman', 'italic']
FontDefinition = t.Tuple[int, FontWeight, FontStyle]
FontValue = t.Tuple['tkinter.font.Font', 'tkinter.Label']

WIDTH = 800
HEIGHT = 600
HSTEP, VSTEP = 13, 18
SCROLL_STEP = 100
FONTS: dict[FontDefinition, FontValue] = {}

# class Text:
#     def __init__(self, text):
#         self.text = text

#     @wbetools.js_hide
#     def __repr__(self):
#         return "Text('{}')".format(self.text)

# class Tag:
#     def __init__(self, tag):
#         self.tag = tag

#     @wbetools.js_hide
#     def __repr__(self):
#         return "Tag('{}')".format(self.tag)


def get_font(size: int, weight: FontWeight, style: FontStyle):
    key = (size, weight, style)

    if key not in FONTS:
        font = tkinter.font.Font(size=size, weight=weight,
            slant=style)
        label = tkinter.Label(font=font)
        FONTS[key] = (font, label)

    return FONTS[key][0]

class Layout:
    def __init__(self, root: 'ewb.PyNode'):
        self.root = root
        self.display_list: t.List[XYTextFont] = []

        self.cursor_x = HSTEP
        self.cursor_y = VSTEP
        self.weight: FontWeight = "normal"
        self.style: FontStyle = "roman"
        self.size = 12

        tokens = self.root.get_all_nodes()

        self.line: t.List[t.Tuple[int, str, 'tkinter.font.Font']] = []

        for tok in tokens:
            self.token(tok)

        # self.flush()

    def token(self, token: 'ewb.PyNode'):
        token_type = token.data.tag_name
        token_content = token.data.attributes.get('content', '')

        if token_type == 'text':
            for word in token_content.split():
                self.word(word)

        self.flush()

        # if isinstance(tok, Text):
        #     for word in tok.text.split():
        #         self.word(word)
        # elif tok.tag == "i":
        #     self.style = "italic"
        # elif tok.tag == "/i":
        #     self.style = "roman"
        # elif tok.tag == "b":
        #     self.weight = "bold"
        # elif tok.tag == "/b":
        #     self.weight = "normal"
        # elif tok.tag == "small":
        #     self.size -= 2
        # elif tok.tag == "/small":
        #     self.size += 2
        # elif tok.tag == "big":
        #     self.size += 4
        # elif tok.tag == "/big":
        #     self.size -= 4
        # elif tok.tag == "br":
        #     self.flush()
        # elif tok.tag == "/p":
        #     self.flush()
        #     self.cursor_y += VSTEP

    def word(self, word: str):
        font = get_font(self.size, self.weight, self.style)
        w = font.measure(word)
        if self.cursor_x + w > WIDTH - HSTEP:
            self.flush()
        self.line.append((self.cursor_x, word, font))
        self.cursor_x += w + font.measure(" ")

    def flush(self):
        if not self.line: return

        metrics = [font.metrics() for _, _, font in self.line]

        max_ascent = max([metric["ascent"] for metric in metrics])
        baseline = self.cursor_y + 1.25 * max_ascent

        for x, word, font in self.line:
            y = baseline - font.metrics("ascent")
            self.display_list.append((x, y, word, font))

        max_descent = max([metric["descent"] for metric in metrics])

        self.cursor_y = baseline + 1.25 * max_descent
        self.cursor_x = HSTEP
        self.line = []


class Browser:
    def __init__(self):
        self.window = tkinter.Tk()
        self.canvas = tkinter.Canvas(
            self.window,
            width=WIDTH,
            height=HEIGHT
        )
        self.canvas.pack()

        self.scroll = 0
        self.window.bind("<Down>", func=self.scrolldown)
        self.display_list = []

    def load(self, url: str):
        body = ewb.request(url)
        root = ewb.load(body)
        self.display_list = Layout(root).display_list
        self.draw()

    def draw(self):
        self.canvas.delete("all")
        for x, y, word, font in self.display_list:
            if y > self.scroll + HEIGHT: continue
            if y + font.metrics("linespace") < self.scroll: continue
            self.canvas.create_text(x, y - self.scroll, text=word, font=font, anchor="nw")

    def scrolldown(self, e: 'tkinter.Event[tkinter.Misc]'):
        self.scroll += SCROLL_STEP
        self.draw()
