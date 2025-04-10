import sys
import typing as t
import ewb
import minify_html

if __name__ == '__main__':
    url = sys.argv[1]

    print(url)

    response = t.cast(str, ewb.request(url))

    # Remove line breaks
    response = minify_html.minify(
        code=response,
        minify_css=True,
        minify_js=True,
        keep_closing_tags=True,
        preserve_brace_template_syntax=True,
        keep_html_and_head_opening_tags=True,
    )

    node = ewb.load(response)

    print(node.data.tag_name)
