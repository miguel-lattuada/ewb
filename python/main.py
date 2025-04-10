import sys
import tkinter
import typing as t

from browser import Browser
import minify_html

import ewb

if __name__ == '__main__':
    url = sys.argv[1]

    # browser = Browser()
    # browser.load(url)

    # tkinter.mainloop()

    # print(url)

    response = t.cast(str, ewb.request(url))

    # # Remove line breaks
    response = minify_html.minify(
        code=response,
        minify_css=True,
        minify_js=True,
        keep_closing_tags=True,
        preserve_brace_template_syntax=True,
        keep_html_and_head_opening_tags=True,
    )

    node = ewb.load(response)
    text_nodes = ewb.find_text_nodes(node)

    for text_node in text_nodes:
        print(text_node.data.attributes)

