const INDEX_BIT_COUNT: usize = 12;
const LENGTH_BIT_COUNT: usize = 4;
const WINDOW_SIZE: usize = 1 << INDEX_BIT_COUNT;
const RAW_LOOK_AHEAD_SIZE: usize = 1 << LENGTH_BIT_COUNT;
const BREAK_EVEN: usize = ((1 + INDEX_BIT_COUNT + LENGTH_BIT_COUNT) / 9) as usize;
const LOOK_AHEAD_SIZE: usize = RAW_LOOK_AHEAD_SIZE + BREAK_EVEN;
const TREE_ROOT: usize = WINDOW_SIZE;
const UNUSED: usize = i32::MAX as usize;

#[derive(Clone, Copy)]
enum Child {
    Smaller = 0,
    Larger = 1,
}

#[derive(Clone)]
struct Node {
    parent: usize,
    children: [usize; 2],
}

impl Node {
    fn new() -> Node {
        Node {
            parent: UNUSED,
            children: [UNUSED; 2],
        }
    }
}

impl From<u8> for Node {
    fn from(value: u8) -> Self {
        Node { parent: value as usize, children: [0, 0] }
    }
}

fn init_tree(tree: &mut [Node]) {
    for node in tree.iter_mut() {
        node.parent = UNUSED;
        node.children = [UNUSED, UNUSED];
    }
    tree[TREE_ROOT].children[Child::Larger as usize] = WINDOW_SIZE + 1;
    tree[WINDOW_SIZE + 1].parent = TREE_ROOT;
}

pub(crate) fn compress(input_data: &[u8]) -> Vec<Node> {
    let mut output_data = Vec::new();
    let mut window = vec![Node::new(); WINDOW_SIZE * 5];
    init_tree(&mut window);
    let mut current_pos = 0;
    let mut match_position = 0;
    let mut len = 0;
    let mut count_bits = 0;
    let mut mask = 1;

    while len < input_data.len() {
        let mut match_length = 0;
        let mut test_node = TREE_ROOT;
        let mut delta = 0;

        loop {
            for i in 0..LOOK_AHEAD_SIZE {
                delta = window[current_pos + i].parent as isize - window[test_node + i].parent as isize;
                if delta != 0 {
                    break;
                }
            }

            if match_length >= LOOK_AHEAD_SIZE {
                replace_node(test_node, current_pos, &mut window);
                break;
            }

            if match_length > 0 && delta == 0 {
                match_length += 1;
            } else {
                match_length = 0;
            }

            let child_prop = if delta >= 0 { Child::Larger } else { Child::Smaller };
            if let Some(child) = get_child(test_node, child_prop, &window) {
                test_node = child;
            } else {
                add_node(test_node, current_pos, child_prop, &mut window);
                break;
            }
        }

        let mut replace_count = 0;
        if match_length <= BREAK_EVEN {
            replace_count = 1;
            output_data.push(mask as u8);
            output_data.push(input_data[current_pos]);
            len += 1;
        } else {
            len += 2;
            let value = ((current_pos - match_position - 1) << LENGTH_BIT_COUNT) | (match_length - BREAK_EVEN - 1);
            let high = (value & 0xFF) as u8;
            let low = ((value >> 8) & 0xFF) as u8;
            output_data.push(high);
            output_data.push(low);
            replace_count = match_length;
        }

        if count_bits == 8 {
            output_data.push(0);
            count_bits = 0;
            mask = 1;
            len += 1;
        } else {
            mask = (mask << 1) & 0xFF;
            count_bits += 1;
        }

        for _ in 0..replace_count {
            delete_node(current_pos, &mut window);
            // Convert input_data[len] to Node and assign it to window
            window[current_pos] = Node::from(input_data[len]);
            len += 1;
            current_pos = (current_pos + 1) % WINDOW_SIZE;
        }
    }

    if count_bits == 0 {
        output_data.pop();
    }

    output_data
}



fn get_child(node: usize, child_prop: Child, window: &Vec<Node>) -> Option<usize> {
    let child_index = child_prop as usize;
    let child = window[node + 2 + child_index];
    if child != UNUSED {
        Some(child)
    } else {
        None
    }
}

fn add_node(parent: usize, child: usize, child_prop: Child, window: &mut Vec<Node>) {
    let child_index = child_prop as usize;
    window[parent + 2 + child_index] = child;
    window[child] = UNUSED;
}

fn replace_node(old_node: usize, new_node: usize, window: &mut Vec<Node>) {
    let parent = window[old_node].parent;
    let parent_node = &mut window[parent];

    if parent_node.children[Child::Smaller as usize] == old_node {
        parent_node.children[Child::Smaller as usize] = new_node;
    } else {
        parent_node.children[Child::Larger as usize] = new_node;
    }

    window[new_node].parent = parent;
    window[old_node].parent = UNUSED;
}

fn delete_node(node: usize, window: &mut Vec<Node>) {
    let parent = window[node];
    if parent == UNUSED {
        return;
    }
    let mut replacement = UNUSED;
    let smaller_child = window[parent + 2];
    let larger_child = window[parent + 3];
    if smaller_child == UNUSED {
        replacement = larger_child;
        contract_node(node, replacement, window);
    } else if larger_child == UNUSED {
        replacement = smaller_child;
        contract_node(node, replacement, window);
    } else {
        replacement = find_next_node(node, window);
        delete_node(replacement, window);
        replace_node(node, replacement, window);
    }
}

fn contract_node(old_node: usize, new_node: usize, window: &mut Vec<usize>) {
    let parent = window[old_node];
    if parent != UNUSED {
        let parent_index = parent;
        if window[parent_index + 2] == old_node {
            window[parent_index + 2] = new_node;
        } else {
            window[parent_index + 3] = new_node;
        }
    }
}

fn find_next_node(node: usize, window: &[usize]) -> usize {
    let mut next = window[node + 3];
    while window[next] != UNUSED {
        next = window[next + 2];
    }
    next
}
