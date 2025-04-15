import typing as t
import tkinter
import tkinter.font

import ewb

XYTextFont = t.Tuple[float, float, str, tkinter.font.Font]
FontWeight = t.Literal['normal', 'bold']
FontStyle = t.Literal['roman', 'italic']
FontDefinition = t.Tuple[int, FontWeight, FontStyle]
FontValue = t.Tuple['tkinter.font.Font', 'tkinter.Label']

WIDTH = 1024
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

        self.flush()

    def token(self, token: 'ewb.PyNode'):
        token_type = token.data.tag_name
        style = self.style
        weight = self.weight
        size = self.size

        # Set content style
        if token_type == "i":
            style = "italic"
        elif token_type == "b":
            weight = "bold"
        elif token_type == "small":
            size -= 2
        elif token_type == "big":
            size += 4
        elif token_type == "h1":
            size += 10

        token_content = token.get_inmidiate_text_node()

        if token_content:
            content = token_content.data.attributes.get('content', '')
            self.word(content, size, weight, style)

        if token_type == "p":
            self.flush()
            self.cursor_y += VSTEP
        if token_type == "h1":
            self.flush()
        elif token_type == "br":
            self.flush()


    def word(self, word: str, size: int, weight: FontWeight, style: FontStyle):
        font = get_font(size, weight, style)
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
