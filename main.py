import typing as t
import ewb
import minify_html

if __name__ == '__main__':
    example_url = "http://example.org"
    response = t.cast(str, ewb.request(example_url))

    # Ignore first line
    response = response.replace('<!doctype html>', '')

    # Remove line breaks
    response = minify_html.minify(
        code=response,
        minify_css=True,
        minify_js=True,
        keep_closing_tags=True,
        preserve_brace_template_syntax=True,
        keep_html_and_head_opening_tags=True,
    )

    node = ewb.load('<html data-darkreader-mode="dynamic" data-darkreader-scheme="dark"><h1 class="title-site">Welcome to my page</h1><h2 class="subtitle-site">Subtitle content</h2></html>')

    print(node)
