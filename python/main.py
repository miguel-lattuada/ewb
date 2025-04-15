import sys
import tkinter
from browser2 import Browser

if __name__ == '__main__':
    url = sys.argv[1]
    Browser().load(url)
    tkinter.mainloop()
