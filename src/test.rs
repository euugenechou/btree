use crate::{node::Node, BTreeMap};

#[test]
fn shuffled() {
    let mut m = BTreeMap::new();
    let shuffled = "iqgkzyrjexbalpcwtsvfmuhdon";

    for c in shuffled.chars() {
        assert_eq!(m.insert(c, c), None);
        assert!(m.contains(&c));
    }

    for (i, c) in shuffled.chars().enumerate() {
        assert_eq!(m.remove(&c), Some(c));

        for (_, d) in shuffled.chars().enumerate().filter(|(j, _)| j <= &i) {
            assert!(!m.contains(&d));
        }

        for (_, d) in shuffled.chars().enumerate().filter(|(j, _)| j > &i) {
            assert!(m.contains(&d));
        }
    }

    assert!(m.is_empty());
}

#[test]
fn inorder() {
    let mut m = BTreeMap::new();

    for i in 0..10 {
        assert_eq!(m.insert(i, i), None);
        assert_eq!(m.insert(i, i + 1), Some(i));
        assert!(m.contains(&i));
        assert!(m.len() == i + 1);
    }

    for i in 0..10 {
        assert_eq!(m.remove(&i), Some(i + 1));
        assert!(m.len() == 9 - i);
    }

    assert!(m.is_empty());
}

#[test]
fn clrs_example_18_8_extended() {
    let mut ab = Node::new();
    ab.keys.push('a');
    ab.keys.push('b');
    ab.vals.push('a');
    ab.vals.push('b');

    let mut def = Node::new();
    def.keys.push('d');
    def.keys.push('e');
    def.keys.push('f');
    def.vals.push('d');
    def.vals.push('e');
    def.vals.push('f');

    let mut jkl = Node::new();
    jkl.keys.push('j');
    jkl.keys.push('k');
    jkl.keys.push('l');
    jkl.vals.push('j');
    jkl.vals.push('k');
    jkl.vals.push('l');

    let mut no = Node::new();
    no.keys.push('n');
    no.keys.push('o');
    no.vals.push('o');
    no.vals.push('n');

    let mut qrs = Node::new();
    qrs.keys.push('q');
    qrs.keys.push('r');
    qrs.keys.push('s');
    qrs.vals.push('q');
    qrs.vals.push('r');
    qrs.vals.push('s');

    let mut uv = Node::new();
    uv.keys.push('u');
    uv.keys.push('v');
    uv.vals.push('u');
    uv.vals.push('v');

    let mut yz = Node::new();
    yz.keys.push('y');
    yz.keys.push('z');
    yz.vals.push('y');
    yz.vals.push('z');

    let mut cgm = Node::new();
    cgm.keys.push('c');
    cgm.keys.push('g');
    cgm.keys.push('m');
    cgm.vals.push('c');
    cgm.vals.push('g');
    cgm.vals.push('m');
    cgm.children.push(ab);
    cgm.children.push(def);
    cgm.children.push(jkl);
    cgm.children.push(no);

    let mut tx = Node::new();
    tx.keys.push('t');
    tx.keys.push('x');
    tx.vals.push('t');
    tx.vals.push('x');
    tx.children.push(qrs);
    tx.children.push(uv);
    tx.children.push(yz);

    let mut p = Node::new();
    p.keys.push('p');
    p.vals.push('p');
    p.children.push(cgm);
    p.children.push(tx);

    let mut m = BTreeMap::with_degree(3);
    m.root = p;
    m.len = 23;

    assert_eq!(
        format!("\n{m:?}"),
        r#"
['p']
├─── ['c', 'g', 'm']
│    ├─── ['a', 'b']
│    ├─── ['d', 'e', 'f']
│    ├─── ['j', 'k', 'l']
│    └─── ['n', 'o']
└─── ['t', 'x']
     ├─── ['q', 'r', 's']
     ├─── ['u', 'v']
     └─── ['y', 'z']
"#
    );

    m.remove(&'f');
    assert_eq!(
        format!("\n{m:?}"),
        r#"
['p']
├─── ['c', 'g', 'm']
│    ├─── ['a', 'b']
│    ├─── ['d', 'e']
│    ├─── ['j', 'k', 'l']
│    └─── ['n', 'o']
└─── ['t', 'x']
     ├─── ['q', 'r', 's']
     ├─── ['u', 'v']
     └─── ['y', 'z']
"#
    );

    m.remove(&'m');
    assert_eq!(
        format!("\n{m:?}"),
        r#"
['p']
├─── ['c', 'g', 'l']
│    ├─── ['a', 'b']
│    ├─── ['d', 'e']
│    ├─── ['j', 'k']
│    └─── ['n', 'o']
└─── ['t', 'x']
     ├─── ['q', 'r', 's']
     ├─── ['u', 'v']
     └─── ['y', 'z']
"#
    );

    m.remove(&'g');
    assert_eq!(
        format!("\n{m:?}"),
        r#"
['p']
├─── ['c', 'l']
│    ├─── ['a', 'b']
│    ├─── ['d', 'e', 'j', 'k']
│    └─── ['n', 'o']
└─── ['t', 'x']
     ├─── ['q', 'r', 's']
     ├─── ['u', 'v']
     └─── ['y', 'z']
"#
    );

    m.remove(&'d');
    assert_eq!(
        format!("\n{m:?}"),
        r#"
['c', 'l', 'p', 't', 'x']
├─── ['a', 'b']
├─── ['e', 'j', 'k']
├─── ['n', 'o']
├─── ['q', 'r', 's']
├─── ['u', 'v']
└─── ['y', 'z']
"#
    );

    m.remove(&'b');
    assert_eq!(
        format!("\n{m:?}"),
        r#"
['e', 'l', 'p', 't', 'x']
├─── ['a', 'c']
├─── ['j', 'k']
├─── ['n', 'o']
├─── ['q', 'r', 's']
├─── ['u', 'v']
└─── ['y', 'z']
"#
    );

    m.remove(&'e');
    assert_eq!(
        format!("\n{m:?}"),
        r#"
['l', 'p', 't', 'x']
├─── ['a', 'c', 'j', 'k']
├─── ['n', 'o']
├─── ['q', 'r', 's']
├─── ['u', 'v']
└─── ['y', 'z']
"#
    );

    m.remove(&'l');
    assert_eq!(
        format!("\n{m:?}"),
        r#"
['k', 'p', 't', 'x']
├─── ['a', 'c', 'j']
├─── ['n', 'o']
├─── ['q', 'r', 's']
├─── ['u', 'v']
└─── ['y', 'z']
"#
    );

    m.remove(&'j');
    assert_eq!(
        format!("\n{m:?}"),
        r#"
['k', 'p', 't', 'x']
├─── ['a', 'c']
├─── ['n', 'o']
├─── ['q', 'r', 's']
├─── ['u', 'v']
└─── ['y', 'z']
"#
    );

    m.remove(&'c');
    assert_eq!(
        format!("\n{m:?}"),
        r#"
['p', 't', 'x']
├─── ['a', 'k', 'n', 'o']
├─── ['q', 'r', 's']
├─── ['u', 'v']
└─── ['y', 'z']
"#
    );

    m.remove(&'y');
    assert_eq!(
        format!("\n{m:?}"),
        r#"
['p', 't']
├─── ['a', 'k', 'n', 'o']
├─── ['q', 'r', 's']
└─── ['u', 'v', 'x', 'z']
"#
    );

    m.remove(&'q');
    assert_eq!(
        format!("\n{m:?}"),
        r#"
['p', 't']
├─── ['a', 'k', 'n', 'o']
├─── ['r', 's']
└─── ['u', 'v', 'x', 'z']
"#
    );

    m.remove(&'t');
    assert_eq!(
        format!("\n{m:?}"),
        r#"
['p', 'u']
├─── ['a', 'k', 'n', 'o']
├─── ['r', 's']
└─── ['v', 'x', 'z']
"#
    );

    m.remove(&'r');
    assert_eq!(
        format!("\n{m:?}"),
        r#"
['o', 'u']
├─── ['a', 'k', 'n']
├─── ['p', 's']
└─── ['v', 'x', 'z']
"#
    );

    m.remove(&'s');
    assert_eq!(
        format!("\n{m:?}"),
        r#"
['n', 'u']
├─── ['a', 'k']
├─── ['o', 'p']
└─── ['v', 'x', 'z']
"#
    );

    m.remove(&'o');
    assert_eq!(
        format!("\n{m:?}"),
        r#"
['n', 'v']
├─── ['a', 'k']
├─── ['p', 'u']
└─── ['x', 'z']
"#
    );

    m.remove(&'x');
    assert_eq!(
        format!("\n{m:?}"),
        r#"
['n']
├─── ['a', 'k']
└─── ['p', 'u', 'v', 'z']
"#
    );

    m.remove(&'a');
    assert_eq!(
        format!("\n{m:?}"),
        r#"
['p']
├─── ['k', 'n']
└─── ['u', 'v', 'z']
"#
    );

    m.remove(&'k');
    assert_eq!(
        format!("\n{m:?}"),
        r#"
['u']
├─── ['n', 'p']
└─── ['v', 'z']
"#
    );

    m.remove(&'u');
    assert_eq!(
        format!("\n{m:?}"),
        r#"
['n', 'p', 'v', 'z']
"#
    );

    m.remove(&'n');
    assert_eq!(
        format!("\n{m:?}"),
        r#"
['p', 'v', 'z']
"#
    );

    m.remove(&'p');
    assert_eq!(
        format!("\n{m:?}"),
        r#"
['v', 'z']
"#
    );

    m.remove(&'v');
    assert_eq!(
        format!("\n{m:?}"),
        r#"
['z']
"#
    );

    m.remove(&'z');
    assert_eq!(
        format!("\n{m:?}"),
        r#"
[]
"#
    );
}
