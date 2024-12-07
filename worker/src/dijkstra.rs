use base::{grids::Grid, Pos};

pub fn get_neighbors<T>(pos: Pos, grid: &Grid<T>) -> Vec<Pos> {
    let x = pos.x;
    let y = pos.y;
    [(x - 1, y), (x + 1, y), (x, y + 1), (x, y - 1)]
        .into_iter()
        .filter(|(x, y)| 0 <= *x && *x < grid.width && 0 <= *y && *y < grid.height)
        .map(|(x, y)| Pos::new(x, y))
        .collect()
}

pub fn dijkstra<F: Fn(Pos) -> i32>(grid: &mut Grid<i32>, seed: &[Pos], cost: F) {
    let mut next: Vec<Pos> = seed
        .iter()
        .flat_map(|pos| get_neighbors(*pos, grid))
        .collect();
    next.extend(seed.iter()); // sometimes its necessary to recompute seeds too

    while !next.is_empty() {
        let buffer: Vec<_> = next.drain(..).collect();
        for pos in buffer.into_iter() {
            let neighbor_max = {
                get_neighbors(pos, grid)
                    .into_iter()
                    .map(|pos| grid.get_clamped(pos.x, pos.y))
                    .max()
                    .cloned()
            };
            if let Some(neighbor_max) = neighbor_max {
                let v = *grid.get_clamped_v(pos);
                let c = cost(pos);
                if neighbor_max > v + c {
                    let new_val = neighbor_max - c;
                    *grid.get_mut(pos.x, pos.y) = new_val;
                    next.extend(
                        get_neighbors(pos, grid)
                            .into_iter()
                            .filter(|pos| *grid.get(pos.x, pos.y) < new_val - cost(*pos)),
                    );
                }
            }
        }
    }
}

/// returns path that follows increasing values until it reaches a local maximium
pub fn dijkstra_path(grid: &Grid<i32>, start: Pos) -> Vec<Pos> {
    let mut path = Vec::new();
    if start.x < 0 || start.y <= 0 || start.x >= grid.width || start.y >= grid.height {
        return path;
    }
    let mut pos = start;
    let mut v = grid[pos];
    if v <= 0 {
        return path;
    }
    path.push(start);
    // do while at home
    while {
        // neighbor with maximum value
        let (npos, nv) = get_neighbors(pos, grid)
            .into_iter()
            .map(|npos| (npos, grid[npos]))
            .max_by_key(|(_, v)| *v)
            .unwrap();
        if nv > v {
            path.push(npos);
            pos = npos;
            v = nv;
            true
        } else {
            false
        }
    } {}
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_neighbors_test() {
        let grid = Grid::new(10, 10, 0);
        let neighbors = get_neighbors(Pos::new(1, 1), &grid);
        assert_eq!(4, neighbors.len());
        assert!(neighbors.contains(&Pos::new(0, 1)));
        assert!(neighbors.contains(&Pos::new(2, 1)));
        assert!(neighbors.contains(&Pos::new(1, 0)));
        assert!(neighbors.contains(&Pos::new(1, 2)));

        let neighbors = get_neighbors(Pos::new(0, 0), &grid);
        assert_eq!(2, neighbors.len());
        assert!(neighbors.contains(&Pos::new(0, 1)));
        assert!(neighbors.contains(&Pos::new(1, 0)));
    }

    #[test]
    fn dijkstra_map_test() {
        // basic
        let mut grid = Grid::new(10, 10, 0);
        let pos = Pos::new(5, 5);
        *grid.get_clamped_mut(pos.x, pos.y) = 5;
        dijkstra(&mut grid, &[pos], |_| 1);
        assert_eq!(2, *grid.get(2, 5));

        // higher cost
        let mut grid = Grid::new(10, 10, 0);
        let pos = Pos::new(5, 5);
        *grid.get_clamped_mut(pos.x, pos.y) = 5;
        dijkstra(&mut grid, &[pos], |_| 2);
        assert_eq!(0, *grid.get(2, 5));
        assert_eq!(1, *grid.get(3, 5));

        // multiple seeds
        let mut grid = Grid::new(10, 10, 0);
        let pos = Pos::new(5, 5);
        *grid.get_clamped_mut(pos.x, pos.y) = 5;
        let pos2 = Pos::new(1, 4);
        *grid.get_clamped_mut(pos2.x, pos2.y) = 5;
        dijkstra(&mut grid, &[pos, pos2], |_| 1);
        assert_eq!(3, *grid.get(2, 5));
    }
}
