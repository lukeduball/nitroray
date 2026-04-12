use std::rc::Rc;

use crate::{math::AxisAlignedBoundingBox};

pub(crate) enum OctreeNodeType<T> {
    OctreeBranch { children: [Option<Rc<OctreeNode<T>>>; 8]},
    OctreeLeaf { contents: Vec<T> },
}

pub(crate) struct OctreeNode<T> {
    axis_aligned_bounding_box: AxisAlignedBoundingBox,
    octree_node_type: OctreeNodeType<T>
}

impl<T> OctreeNode<T> {
    pub(crate) fn new_leaf_node(axis_aligned_bounding_box: AxisAlignedBoundingBox, contents: Vec<T>) -> Self {
        Self {
            axis_aligned_bounding_box,
            octree_node_type: OctreeNodeType::OctreeLeaf { contents }
        }
    }

    pub(crate) fn new_branch_node(axis_aligned_bounding_box: AxisAlignedBoundingBox) -> Self {
        Self {
            axis_aligned_bounding_box,
            octree_node_type: OctreeNodeType::OctreeBranch { children: [const {None}; 8] }
        }
    }
}

pub(crate) struct Octree<T> {
    minimum_leaves: u32,
    maximum_depth: u32,
    root: Rc<OctreeNode<T>>
}

impl<T> Octree<T> {
    pub(crate) fn new(minimum_leaves: u32, maximum_depth: u32, root: OctreeNode<T>) -> Octree<T> {
        Self {
            minimum_leaves,
            maximum_depth,
            root: Rc::new(root)
        }
    }

    pub(crate) fn get_root_axis_aligned_bounding_box(&self) -> AxisAlignedBoundingBox {
        self.root.axis_aligned_bounding_box
    }
}