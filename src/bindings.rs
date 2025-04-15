use std::collections::HashMap;

use crate::html::{HTMLParser, Node, NodeData};
use crate::url::{URLError, URL};

use pyo3::prelude::*;
use pyo3::{exceptions::PyValueError, pyfunction, PyResult};

#[pyclass]
#[derive(Clone)]
pub struct PyNodeData {
    #[pyo3(get)]
    pub tag_name: String,
    #[pyo3(get)]
    pub attributes: HashMap<String, String>,
}

#[pyclass]
#[derive(Clone)]
pub struct PyNode {
    #[pyo3(get)]
    pub children: Vec<PyNode>,
    #[pyo3(get)]
    pub data: PyNodeData,
}

#[pymethods]
impl PyNode {
    fn get_text_nodes(&self) -> PyResult<Vec<PyNode>> {
        Ok(self.get_nodes("text"))
    }

    fn get_nodes(&self, node_type: &str) -> Vec<PyNode> {
        let mut res = Vec::new();

        for child in &self.children {
            if child.data.tag_name == node_type {
                res.push(child.clone());
            } else {
                res.extend(child.get_nodes(node_type));
            }
        }

        res
    }
}

impl From<&Node> for PyNode {
    fn from(value: &Node) -> Self {
        Self {
            children: value.children.iter().map(PyNode::from).collect(),
            data: PyNodeData {
                tag_name: value.data.tag_name.clone(),
                attributes: value.data.attributes.clone(),
            },
        }
    }
}

impl Into<Node> for &PyNode {
    fn into(self) -> Node {
        Node {
            children: self.children.iter().map(|node| node.into()).collect(),
            data: NodeData {
                tag_name: self.data.tag_name.clone(),
                attributes: self.data.attributes.clone(),
            },
        }
    }
}

#[pyfunction]
pub fn request(url: &str) -> PyResult<String> {
    let mut url_intent = URL::new(url.to_string());

    match &mut url_intent {
        Ok(url) => {
            if let Ok(response) = url.request() {
                Ok(response.clone())
            } else {
                Err(PyValueError::new_err("Error: unable to send request"))
            }
        }
        Err(error) => {
            // potential issue we downcast to something else
            if let Some(url_error) = error.downcast_ref::<URLError>() {
                Err(PyValueError::new_err(url_error.message.clone()))
            } else {
                Err(PyValueError::new_err(
                    "Error: unable to create URL instance",
                ))
            }
        }
    }
}

#[pyfunction]
pub fn load(body: &str) -> PyResult<PyNode> {
    let mut parser = HTMLParser::new(body);
    let root = parser.parse().unwrap();

    Ok(PyNode::from(&root))
}

#[pyfunction]
pub fn find_text_nodes(pynode: &PyNode) -> PyResult<Vec<PyNode>> {
    let node: Node = pynode.into();
    let text_nodes = node.find_text_nodes();

    Ok(text_nodes
        .iter()
        .map(|n| PyNode::from(n.to_owned()))
        .collect())
}
