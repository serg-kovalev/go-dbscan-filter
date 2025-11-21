//! This code is heavily based on <https://godoc.org/code.google.com/p/eaburns/kdtree>
//!
//! Original code is under New BSD License.
//! Author: Ethan Burns <burns.ethan@gmail.com>

use super::point::{Point, PointList};

/// KD-Tree implementation for efficient spatial queries
///
/// Points are separated from nodes. Nodes hold only indices into the Points slice.
pub struct KDTree {
    /// All points in the tree
    pub points: PointList,
    /// Root node of the tree
    pub root: Option<Box<KDTreeNode>>,
}

/// A node in the K-D tree
pub struct KDTreeNode {
    /// Index of the point associated with this node
    pub point_id: usize,
    /// Indices of points equal to this node's point
    pub equal_ids: Vec<usize>,

    split: usize,
    left: Option<Box<KDTreeNode>>,
    right: Option<Box<KDTreeNode>>,
}

impl KDTree {
    /// Inserts a point into the K-D tree
    ///
    /// Inserting a node that is already a member of a K-D tree invalidates that tree.
    #[allow(dead_code)] // Part of public API, may be used by external code
    pub fn insert(&mut self, point: Point) {
        self.points.push(point);
        let point_id = self.points.len() - 1;
        let new_node = KDTreeNode {
            point_id,
            equal_ids: Vec::new(),
            split: 0,
            left: None,
            right: None,
        };
        let root = self.root.take();
        self.root = Some(Box::new(self.insert_node(root, 0, new_node)));
    }

    fn insert_node(
        &self,
        t: Option<Box<KDTreeNode>>,
        depth: usize,
        mut n: KDTreeNode,
    ) -> KDTreeNode {
        match t {
            None => {
                n.split = depth % 2;
                n
            }
            Some(mut t_node) => {
                if self.points[n.point_id].0[t_node.split]
                    < self.points[t_node.point_id].0[t_node.split]
                {
                    t_node.left =
                        Some(Box::new(self.insert_node(t_node.left.take(), depth + 1, n)));
                } else {
                    t_node.right = Some(Box::new(self.insert_node(
                        t_node.right.take(),
                        depth + 1,
                        n,
                    )));
                }
                *t_node
            }
        }
    }

    /// Finds all nodes in the K-D tree that are within a given distance from the given point
    ///
    /// To avoid allocation, the `nodes` vector can be pre-allocated with a larger
    /// capacity and re-used across multiple calls.
    pub fn in_range(&self, pt: &Point, dist: f64, mut nodes: Vec<usize>) -> Vec<usize> {
        if dist < 0.0 {
            return nodes;
        }
        self.in_range_recursive(self.root.as_deref(), pt, dist, &mut nodes);
        nodes
    }

    fn in_range_recursive(
        &self,
        t: Option<&KDTreeNode>,
        pt: &Point,
        r: f64,
        nodes: &mut Vec<usize>,
    ) {
        let t = match t {
            None => return,
            Some(t) => t,
        };

        let diff = pt.0[t.split] - self.points[t.point_id].0[t.split];

        let (this_side, other_side) = if diff < 0.0 {
            (t.left.as_deref(), t.right.as_deref())
        } else {
            (t.right.as_deref(), t.left.as_deref())
        };

        let mut p1 = Point([0.0, 0.0]);
        p1.0[1 - t.split] = (pt.0[1 - t.split] + self.points[t.point_id].0[1 - t.split]) / 2.0;
        p1.0[t.split] = pt.0[t.split];

        let mut p2 = Point([0.0, 0.0]);
        p2.0[1 - t.split] = (pt.0[1 - t.split] + self.points[t.point_id].0[1 - t.split]) / 2.0;
        p2.0[t.split] = self.points[t.point_id].0[t.split];

        let dist = p1.sq_dist(&p2);

        self.in_range_recursive(this_side, pt, r, nodes);
        if dist <= r * r {
            if self.points[t.point_id].sq_dist(pt) < r * r {
                nodes.push(t.point_id);
                nodes.extend_from_slice(&t.equal_ids);
            }
            self.in_range_recursive(other_side, pt, r, nodes);
        }
    }

    /// Returns the height of the K-D tree
    #[allow(dead_code)] // Part of public API, may be used by external code
    pub fn height(&self) -> usize {
        self.root.as_ref().map_or(0, |r| r.height())
    }
}

impl KDTreeNode {
    fn height(&self) -> usize {
        let ht = self.left.as_ref().map_or(0, |l| l.height());
        let rht = self.right.as_ref().map_or(0, |r| r.height());
        ht.max(rht) + 1
    }
}

/// Creates a new K-D tree built from the given points
pub fn new_kd_tree(points: PointList) -> KDTree {
    let mut result = KDTree { points, root: None };

    if !result.points.is_empty() {
        result.root = build_tree(0, &pre_sort(&result.points));
    }

    result
}

/// Builds a tree node by finding the median point and recursively building left and right subtrees
fn build_tree(depth: usize, nodes: &PreSorted) -> Option<Box<KDTreeNode>> {
    let split = depth % 2;
    match nodes.cur[split].len() {
        0 => None,
        1 => Some(Box::new(KDTreeNode {
            point_id: nodes.cur[split][0],
            equal_ids: Vec::new(),
            split,
            left: None,
            right: None,
        })),
        _ => {
            let (med, equal, left, right) = nodes.split_med(split);
            Some(Box::new(KDTreeNode {
                point_id: med,
                equal_ids: equal,
                split,
                left: build_tree(depth + 1, &left),
                right: build_tree(depth + 1, &right),
            }))
        }
    }
}

/// Holds nodes pre-sorted on each dimension
struct PreSorted {
    points: PointList,
    /// Currently sorted set of point IDs by dimension
    cur: [Vec<usize>; 2],
}

/// Pre-sorts nodes on each dimension
fn pre_sort(points: &PointList) -> PreSorted {
    let mut p = PreSorted {
        points: points.clone(),
        cur: [Vec::new(), Vec::new()],
    };
    for i in 0..2 {
        p.cur[i] = (0..points.len()).collect();
        p.cur[i].sort_by(|&a, &b| {
            let a_val = points[a].0[i];
            let b_val = points[b].0[i];
            if a_val == b_val {
                // For equal values, sort by the other dimension
                // Use unwrap_or_else to handle NaN (though shouldn't occur in valid geo data)
                points[a].0[1 - i]
                    .partial_cmp(&points[b].0[1 - i])
                    .unwrap_or(std::cmp::Ordering::Equal)
            } else {
                a_val
                    .partial_cmp(&b_val)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }
        });
    }
    p
}

impl PreSorted {
    /// Returns the median node on the split dimension and two PreSorted structs
    /// that contain the nodes (still sorted on each dimension) that are less than
    /// and greater than or equal to the median node value on the given splitting dimension.
    fn split_med(&self, dim: usize) -> (usize, Vec<usize>, PreSorted, PreSorted) {
        let mut m = self.cur[dim].len() / 2;
        while m > 0
            && self.points[self.cur[dim][m - 1]].0[dim] == self.points[self.cur[dim][m]].0[dim]
        {
            m -= 1;
        }
        let mut mh = m;
        while mh < self.cur[dim].len() - 1
            && self.points[self.cur[dim][mh + 1]] == self.points[self.cur[dim][m]]
        {
            mh += 1;
        }
        let med = self.cur[dim][m];
        let equal = self.cur[dim][m + 1..=mh].to_vec();
        let pivot = self.points[med].0[dim];

        let mut left = PreSorted {
            points: self.points.clone(),
            cur: [Vec::new(), Vec::new()],
        };
        left.cur[dim] = self.cur[dim][..m].to_vec();

        let mut right = PreSorted {
            points: self.points.clone(),
            cur: [Vec::new(), Vec::new()],
        };
        right.cur[dim] = self.cur[dim][mh + 1..].to_vec();

        for d in 0..2 {
            if d == dim {
                continue;
            }

            left.cur[d] = Vec::with_capacity(self.cur[d].len());
            right.cur[d] = Vec::with_capacity(self.cur[d].len());

            for &n in &self.cur[d] {
                if n == med {
                    continue;
                }
                let mut skip = false;
                for &x in &equal {
                    if n == x {
                        skip = true;
                        break;
                    }
                }
                if skip {
                    continue;
                }
                if self.points[n].0[dim] < pivot {
                    left.cur[d].push(n);
                } else {
                    right.cur[d].push(n);
                }
            }
        }

        (med, equal, left, right)
    }
}

// Re-export with Go-style name
pub use new_kd_tree as NewKDTree;
