import typing as t

class PyNodeData():
    tag_name: str
    attributes: t.Dict[str, str]

class PyNode:
    children: t.List[PyNode]
    data: PyNodeData
    def get_text_nodes(self) -> t.List[PyNode]: ...
    def get_nodes(self, node_type: str) -> t.List[PyNode]: ...

def request(url: str) -> str: ...
def load(body: str) -> PyNode: ...
