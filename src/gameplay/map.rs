//! Floor plan traced in a 640-wide reference frame from assets/mapa.png.
//! Rectangles cover rooms, doorways and corridors; all consumers use this union.
use super::geometry::Point;
use std::{cell::RefCell, collections::VecDeque, sync::LazyLock};

pub(super) const COLS: usize = 160;
pub(super) const ROWS: usize = 96;
pub(super) const CELL: f32 = 16.;
const COUNT: usize = COLS * ROWS;
const FLOOR: &[(usize, usize, usize, usize)] = &[
    // Northwest chamber group and its two approaches
    (116,30,160,85),(158,36,181,49),(180,30,216,69),(191,67,206,99),
    (131,82,144,141),(131,96,218,110),(110,139,185,179),(140,177,154,196),
    (215,92,248,130),(223,127,238,167),(198,150,239,165),(198,162,212,210),
    (233,150,285,165),(273,160,286,210),
    // Western loop and central approach
    (35,153,84,197),(51,193,66,246),(59,214,117,229),(29,243,84,301),
    (80,287,115,301),(112,271,196,333),(142,249,155,274),
    (113,193,198,253),(194,210,222,226),(220,186,273,262),
    (238,260,257,299),(194,296,279,309),(271,207,299,222),
    // North vaults
    (265,48,330,109),(306,103,330,151),(306,128,375,162),
    (327,66,376,83),(333,22,360,61),(345,57,356,71),
    (358,28,376,42),(373,22,471,86),(411,83,430,101),
    // Central hall and eastern loop
    (335,160,352,181),(296,178,383,243),(336,240,353,264),
    (374,142,402,158),(385,139,401,193),(380,182,401,199),
    (400,98,450,200),(447,159,473,173),(470,137,517,179),
    (421,198,437,217),(392,215,450,296),(412,289,448,315),
    (449,239,475,253),(462,215,478,250),(465,216,541,231),
    (527,158,540,222),(515,158,569,172),(556,128,569,163),
    (539,83,584,119),(547,76,576,129),(532,94,591,113),
    (537,201,562,214),(559,182,601,237),(577,233,591,307),
    (541,293,585,307),(477,269,540,333),(536,291,563,310),
    // Southern galleries
    (447,295,480,308),(415,312,428,352),(384,340,424,353),
    (301,262,380,299),(276,294,390,354),(270,311,281,354),
    (378,282,395,294),(379,223,396,235),
];

static GRID: LazyLock<Vec<bool>> = LazyLock::new(|| {
    (0..COUNT).map(|i| {
        let (x,y) = ((i % COLS)*4+2, (i / COLS)*4+2);
        FLOOR.iter().any(|&(l,t,r,b)| x >= l && x < r && y >= t && y < b)
    }).collect()
});
pub(super) fn floor(x: usize, y: usize) -> bool {
    x < COLS && y < ROWS && GRID[y*COLS+x]
}
fn index(p: Point) -> usize {
    (p.y / CELL).clamp(0., (ROWS-1) as f32) as usize * COLS
        + (p.x / CELL).clamp(0., (COLS-1) as f32) as usize
}
fn center(i: usize) -> Point {
    Point::at(((i%COLS) as f32+0.5)*CELL, ((i/COLS) as f32+0.5)*CELL)
}
pub(super) fn on_floor(p: Point) -> Point {
    if GRID[index(p)] { return p; }
    let nearest = (0..COUNT).filter(|&i| GRID[i]).min_by_key(|&i| {
        let d = center(i)-p;
        (d.x*d.x+d.y*d.y) as u64
    }).unwrap();
    center(nearest)
}
fn neighbors(i: usize) -> impl Iterator<Item=usize> {
    [i.wrapping_sub(1), i+1, i.wrapping_sub(COLS), i+COLS]
        .into_iter().filter(move |&j| j<COUNT && GRID[j] && (j%COLS).abs_diff(i%COLS)<=1)
}
fn routes(goal: usize) -> Vec<usize> {
    let mut next = vec![usize::MAX; COUNT];
    let mut queue = VecDeque::from([goal]);
    next[goal] = goal;
    while let Some(i) = queue.pop_front() {
        for j in neighbors(i) {
            if next[j] == usize::MAX { next[j]=i; queue.push_back(j); }
        }
    }
    next
}
// Test the crossed cells in sub-cell steps, including both sides at a diagonal corner.
pub(super) fn clear_segment(from: Point, to: Point) -> Point {
    let delta = to-from;
    let steps = (delta.length()/(CELL*0.24)).ceil().max(1.) as usize;
    let mut last = from;
    for k in 1..=steps {
        let p=from+delta*(k as f32/steps as f32);
        if !GRID[index(p)] || !GRID[index(Point::at(last.x,p.y))] || !GRID[index(Point::at(p.x,last.y))] { break; }
        last=p;
    }
    last
}
thread_local! { static ROUTES: RefCell<VecDeque<(usize,Vec<usize>)>> = RefCell::default(); }
pub(super) fn waypoint(from: Point, to: Point) -> Point {
    if from.distance(to)<0.01 { return from; }
    let to=on_floor(to);
    if clear_segment(from,to).distance(to)<0.01 { return to; }
    let goal=index(to);
    ROUTES.with_borrow_mut(|cache| {
        if !cache.iter().any(|(g,_)| *g==goal) {
            if cache.len()>=4 { cache.pop_front(); }
            cache.push_back((goal,routes(goal)));
        }
        let next=&cache.iter().find(|(g,_)| *g==goal).unwrap().1;
        let mut cell=index(from);
        let mut target=from;
        for _ in 0..12 {
            let j=next[cell];
            if j==usize::MAX || j==cell {break;}
            let point=center(j);
            if clear_segment(from,point).distance(point)>0.01 {break;}
            target=point;
            cell=j;
        }
        target
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn a_unit_walks_through_corridors_between_objectives() {
        let mut current=Point::at(157.*4.,221.*4.);
        let goal=Point::at(509.*4.,301.*4.);
        for _ in 0..2400 {
            let delta=waypoint(current,goal)-current;
            current=current+delta.direction()*delta.length().min(5.);
            assert!(GRID[index(current)]);
            if current.distance(goal)<1. {return;}
        }
        panic!("unit did not reach the eastern objective");
    }
    #[test]
    fn all_rooms_connect_to_the_western_objective() {
        let paths=routes(index(Point::at(157.*4.,221.*4.)));
        assert!((0..COUNT).all(|i| !GRID[i] || paths[i]!=usize::MAX));
    }
}
