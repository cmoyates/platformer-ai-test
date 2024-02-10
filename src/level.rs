use bevy::{math::Vec2, render::color::Color};
use rand::Rng;

use crate::utils::line_intersect;

pub struct Polygon {
    pub points: Vec<Vec2>,
    pub color: Color,
    pub is_container: bool,
}

const LEVEL_DATA: &'static [u8] = include_bytes!("../assets/level.json");

pub fn generate_level_polygons(grid_size: f32) -> (Vec<Polygon>, Vec2, Vec2) {
    let mut rng = rand::thread_rng();

    let res = std::str::from_utf8(LEVEL_DATA);
    let level_grid_data: Vec<Vec<usize>> = serde_json::from_str(&res.unwrap()).unwrap();

    let size = Vec2::new(
        level_grid_data[0].len() as f32,
        level_grid_data.len() as f32,
    );

    let offset = Vec2::new(size.x * -grid_size / 2.0, size.y * grid_size / 2.0);

    let mut line_points = get_line_points(level_grid_data, grid_size, size);

    let mut line_count = line_points.len() / 2;

    // Remove superfluous points

    let mut point_removal_data = Some(((0, 0), (0, 0)));

    // While there are points to remove
    while point_removal_data.is_some() {
        point_removal_data = None;

        'outer: for i in 0..line_count {
            for j in 0..line_count {
                // If the lines are the same, skip
                if i == j {
                    continue;
                }

                // Check if either of the points are shared

                let line_1_start = line_points[i * 2];
                let line_1_end = line_points[i * 2 + 1];

                let line_2_start = line_points[j * 2];
                let line_2_end = line_points[j * 2 + 1];

                let mut shared_point: Option<(usize, usize)> = None;
                let mut unique_points: Option<(usize, usize)> = None;

                if line_1_start == line_2_start {
                    shared_point = Some((i * 2, j * 2));
                    unique_points = Some((i * 2 + 1, j * 2 + 1));
                } else if line_1_start == line_2_end {
                    shared_point = Some((i * 2, j * 2 + 1));
                    unique_points = Some((i * 2 + 1, j * 2));
                } else if line_1_end == line_2_start {
                    shared_point = Some((i * 2 + 1, j * 2));
                    unique_points = Some((i * 2, j * 2 + 1));
                } else if line_1_end == line_2_end {
                    shared_point = Some((i * 2 + 1, j * 2 + 1));
                    unique_points = Some((i * 2, j * 2));
                }

                // If there is no shared point, skip
                if shared_point.is_none() {
                    continue;
                }

                // Check if the lines are parallel

                let dot = (line_1_start - line_1_end)
                    .normalize()
                    .dot((line_2_start - line_2_end).normalize());
                if dot.abs() == 1.0 {
                    // if so flag the point for removal and break out of the outer for loop
                    point_removal_data = Some((shared_point.unwrap(), unique_points.unwrap()));
                    break 'outer;
                }
            }
        }

        // If there is a point to remove
        if point_removal_data.is_some() {
            let point_removal_data = point_removal_data.unwrap();

            // Store the unique vertices
            let unique_vert_1 = line_points[point_removal_data.1 .0].clone();
            let unique_vert_2 = line_points[point_removal_data.1 .1].clone();

            // Remove the shared and unique vertices
            let mut removal_indices = vec![
                point_removal_data.0 .0,
                point_removal_data.0 .1,
                point_removal_data.1 .0,
                point_removal_data.1 .1,
            ];
            removal_indices.sort();
            removal_indices.reverse();
            for i in removal_indices {
                line_points.remove(i);
            }

            // Add the unique vertices back
            line_points.push(unique_vert_1);
            line_points.push(unique_vert_2);

            // Update the line count
            line_count -= 1;
        }
    }

    for i in 0..line_points.len() {
        line_points[i].x += offset.x;
        line_points[i].y *= -1.0;
        line_points[i].y += offset.y;
    }

    // Separate the lines into polygons
    let mut polygons: Vec<Polygon> = Vec::new();

    let container_color = Color::rgb(
        rng.gen_range(0.0..=1.0),
        rng.gen_range(0.0..=1.0),
        rng.gen_range(0.0..=1.0),
    );

    // While there are lines left
    while line_count > 0 {
        // Create a new polygon
        let mut polygon_lines: Vec<Vec2> = Vec::new();

        // Add the first line to the polygon
        polygon_lines.push(line_points[0].clone());
        polygon_lines.push(line_points[1].clone());

        // Remove the first line from the list of lines
        line_points.remove(0);
        line_points.remove(0);

        // Decrement the line count
        line_count -= 1;

        let start_vert = polygon_lines[0].clone();
        let mut current_vert = polygon_lines[polygon_lines.len() - 1].clone();

        // While the polygon is not closed
        while start_vert != current_vert {
            for i in 0..line_count {
                let line_start = line_points[i * 2].clone();
                let line_end = line_points[i * 2 + 1].clone();

                // If the line starts at the current vertex
                if line_start == current_vert {
                    // Add the line to the polygon
                    polygon_lines.push(line_end.clone());

                    // Remove the line from the list of lines
                    line_points.remove(i * 2);
                    line_points.remove(i * 2);

                    // Decrement the line count
                    line_count -= 1;

                    // Set the current vertex to the end of the line
                    current_vert = line_end;

                    // Break out of the for loop
                    break;
                }
                // If the line ends at the current vertex
                else if line_end == current_vert {
                    // Add the line to the polygon
                    polygon_lines.push(line_start.clone());

                    // Remove the line from the list of lines
                    line_points.remove(i * 2);
                    line_points.remove(i * 2);

                    // Decrement the line count
                    line_count -= 1;

                    // Set the current vertex to the start of the line
                    current_vert = line_start;

                    // Break out of the for loop
                    break;
                }
            }
        }

        let is_container = point_in_polygon(&polygon_lines, Vec2::new(0.0, 0.0));
        let mut collision_side = calculate_winding_order(&polygon_lines).signum();
        if is_container {
            collision_side *= -1.0;
        }

        if collision_side == -1.0 {
            polygon_lines.reverse();
        }

        let color = if is_container {
            container_color
        } else {
            Color::rgb(
                rng.gen_range(0.0..=1.0),
                rng.gen_range(0.0..=1.0),
                rng.gen_range(0.0..=1.0),
            )
        };

        // Add the polygon to the list of polygons
        polygons.push(Polygon {
            points: polygon_lines,
            color,
            is_container,
        });
    }

    return (polygons, size, size / 2.0);
}

fn calculate_winding_order(vertices: &Vec<Vec2>) -> f32 {
    let mut sum = 0.0;

    for i in 0..vertices.len() {
        let p1 = vertices[i];
        let p2 = vertices[(i + 1) % vertices.len()];
        sum += (p2.x - p1.x) * (p2.y + p1.y);
    }

    sum
}

fn point_in_polygon(polygon_lines: &Vec<Vec2>, point: Vec2) -> bool {
    let test_line_start = point;
    let test_line_end = point + Vec2::new(2.0, 1.0) * 1000.0;

    let mut intersect_counter = 0;

    for i in 1..polygon_lines.len() {
        let start = polygon_lines[i - 1];
        let end = polygon_lines[i];

        let intersection = line_intersect(start, end, test_line_start, test_line_end);

        if intersection.is_some() {
            intersect_counter += 1;
        }
    }

    return intersect_counter % 2 == 1;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolygonLine {
    pub polygon_index: usize,
    pub line_index: usize,
}

fn get_line_points(level_grid_data: Vec<Vec<usize>>, grid_cell_size: f32, size: Vec2) -> Vec<Vec2> {
    let mut line_points: Vec<Vec2> = Vec::new();

    for y in 0..size.y as usize {
        for x in 0..size.x as usize {
            let tile = level_grid_data[y][x];

            match tile {
                1 => {
                    // Squares

                    // Left edge
                    if x == 0 || level_grid_data[y][x - 1] == 0 {
                        line_points.push(Vec2::new(
                            x as f32 * grid_cell_size,
                            y as f32 * grid_cell_size,
                        ));
                        line_points.push(Vec2::new(
                            x as f32 * grid_cell_size,
                            (y + 1) as f32 * grid_cell_size,
                        ));
                    }
                    // Right edge
                    if x == level_grid_data[y].len() - 1 || level_grid_data[y][x + 1] == 0 {
                        line_points.push(Vec2::new(
                            (x + 1) as f32 * grid_cell_size,
                            y as f32 * grid_cell_size,
                        ));
                        line_points.push(Vec2::new(
                            (x + 1) as f32 * grid_cell_size,
                            (y + 1) as f32 * grid_cell_size,
                        ));
                    }
                    // Top edge
                    if y == 0 || level_grid_data[y - 1][x] == 0 {
                        line_points.push(Vec2::new(
                            x as f32 * grid_cell_size,
                            y as f32 * grid_cell_size,
                        ));
                        line_points.push(Vec2::new(
                            (x + 1) as f32 * grid_cell_size,
                            y as f32 * grid_cell_size,
                        ));
                    }
                    // Bottom edge
                    if y == size.y as usize - 1 || level_grid_data[y + 1][x] == 0 {
                        line_points.push(Vec2::new(
                            x as f32 * grid_cell_size,
                            (y + 1) as f32 * grid_cell_size,
                        ));
                        line_points.push(Vec2::new(
                            (x + 1) as f32 * grid_cell_size,
                            (y + 1) as f32 * grid_cell_size,
                        ));
                    }
                }
                2..=5 => {
                    // Right triangles

                    let triangle_type = tile - 2;

                    match triangle_type {
                        0 => {
                            // Bottom left

                            // Hypotenuse
                            line_points.push(Vec2::new(
                                x as f32 * grid_cell_size,
                                y as f32 * grid_cell_size,
                            ));
                            line_points.push(Vec2::new(
                                (x + 1) as f32 * grid_cell_size,
                                (y + 1) as f32 * grid_cell_size,
                            ));

                            // Bottom edge
                            if y == size.y as usize - 1 || level_grid_data[y + 1][x] == 0 {
                                line_points.push(Vec2::new(
                                    x as f32 * grid_cell_size,
                                    (y + 1) as f32 * grid_cell_size,
                                ));
                                line_points.push(Vec2::new(
                                    (x + 1) as f32 * grid_cell_size,
                                    (y + 1) as f32 * grid_cell_size,
                                ));
                            }

                            // Left edge
                            if x == 0 || level_grid_data[y][x - 1] == 0 {
                                line_points.push(Vec2::new(
                                    x as f32 * grid_cell_size,
                                    y as f32 * grid_cell_size,
                                ));
                                line_points.push(Vec2::new(
                                    x as f32 * grid_cell_size,
                                    (y + 1) as f32 * grid_cell_size,
                                ));
                            }
                        }
                        1 => {
                            // Bottom right

                            // Hypotenuse
                            line_points.push(Vec2::new(
                                (x + 1) as f32 * grid_cell_size,
                                y as f32 * grid_cell_size,
                            ));
                            line_points.push(Vec2::new(
                                x as f32 * grid_cell_size,
                                (y + 1) as f32 * grid_cell_size,
                            ));

                            // Bottom edge
                            if y == size.y as usize - 1 || level_grid_data[y + 1][x] == 0 {
                                line_points.push(Vec2::new(
                                    x as f32 * grid_cell_size,
                                    (y + 1) as f32 * grid_cell_size,
                                ));
                                line_points.push(Vec2::new(
                                    (x + 1) as f32 * grid_cell_size,
                                    (y + 1) as f32 * grid_cell_size,
                                ));
                            }

                            // Right edge
                            if x == level_grid_data[y].len() - 1 || level_grid_data[y][x + 1] == 0 {
                                line_points.push(Vec2::new(
                                    (x + 1) as f32 * grid_cell_size,
                                    y as f32 * grid_cell_size,
                                ));
                                line_points.push(Vec2::new(
                                    (x + 1) as f32 * grid_cell_size,
                                    (y + 1) as f32 * grid_cell_size,
                                ));
                            }
                        }
                        2 => {
                            // Top left

                            // Hypotenuse
                            line_points.push(Vec2::new(
                                x as f32 * grid_cell_size,
                                (y + 1) as f32 * grid_cell_size,
                            ));
                            line_points.push(Vec2::new(
                                (x + 1) as f32 * grid_cell_size,
                                y as f32 * grid_cell_size,
                            ));

                            // Top edge
                            if y == 0 || level_grid_data[y - 1][x] == 0 {
                                line_points.push(Vec2::new(
                                    x as f32 * grid_cell_size,
                                    y as f32 * grid_cell_size,
                                ));
                                line_points.push(Vec2::new(
                                    (x + 1) as f32 * grid_cell_size,
                                    y as f32 * grid_cell_size,
                                ));
                            }

                            // Left edge
                            if x == 0 || level_grid_data[y][x - 1] == 0 {
                                line_points.push(Vec2::new(
                                    x as f32 * grid_cell_size,
                                    y as f32 * grid_cell_size,
                                ));
                                line_points.push(Vec2::new(
                                    x as f32 * grid_cell_size,
                                    (y + 1) as f32 * grid_cell_size,
                                ));
                            }
                        }
                        3 => {
                            // Top right

                            // Hypotenuse
                            line_points.push(Vec2::new(
                                (x + 1) as f32 * grid_cell_size,
                                (y + 1) as f32 * grid_cell_size,
                            ));
                            line_points.push(Vec2::new(
                                x as f32 * grid_cell_size,
                                y as f32 * grid_cell_size,
                            ));

                            // Top edge
                            if y == 0 || level_grid_data[y - 1][x] == 0 {
                                line_points.push(Vec2::new(
                                    x as f32 * grid_cell_size,
                                    y as f32 * grid_cell_size,
                                ));
                                line_points.push(Vec2::new(
                                    (x + 1) as f32 * grid_cell_size,
                                    y as f32 * grid_cell_size,
                                ));
                            }

                            // Right edge
                            if x == level_grid_data[y].len() - 1 || level_grid_data[y][x + 1] == 0 {
                                line_points.push(Vec2::new(
                                    (x + 1) as f32 * grid_cell_size,
                                    y as f32 * grid_cell_size,
                                ));
                                line_points.push(Vec2::new(
                                    (x + 1) as f32 * grid_cell_size,
                                    (y + 1) as f32 * grid_cell_size,
                                ));
                            }
                        }
                        _ => {}
                    }
                }
                6..=9 => {
                    // // Isosceles triangles

                    // let triangle_type = tile - 6;

                    // match triangle_type {
                    //     0 => {
                    //         // Bottom

                    //         // Top left
                    //         line_points.push(Vertex::with_pos_color(
                    //             Vec2::new(x as f32 * grid_size, (y + 1) as f32 * grid_size),
                    //             Color::RED,
                    //         ));
                    //         line_points.push(Vertex::with_pos_color(
                    //             Vec2::new(
                    //                 (x as f32 + 0.5) as f32 * grid_size,
                    //                 (y as f32 + 0.5) as f32 * grid_size,
                    //             ),
                    //             Color::RED,
                    //         ));

                    //         // Top right
                    //         line_points.push(Vertex::with_pos_color(
                    //             Vec2::new(
                    //                 (x as f32 + 1.0) as f32 * grid_size,
                    //                 (y + 1) as f32 * grid_size,
                    //             ),
                    //             Color::RED,
                    //         ));
                    //         line_points.push(Vertex::with_pos_color(
                    //             Vec2::new(
                    //                 (x as f32 + 0.5) as f32 * grid_size,
                    //                 (y as f32 + 0.5) as f32 * grid_size,
                    //             ),
                    //             Color::RED,
                    //         ));

                    //         // Bottom edge
                    //         if y == json_data.len() - 1 || json_data[y + 1][x] == 0 {
                    //             line_points.push(Vertex::with_pos_color(
                    //                 Vec2::new(x as f32 * grid_size, (y + 1) as f32 * grid_size),
                    //                 Color::RED,
                    //             ));
                    //             line_points.push(Vertex::with_pos_color(
                    //                 Vec2::new(
                    //                     (x + 1) as f32 * grid_size,
                    //                     (y + 1) as f32 * grid_size,
                    //                 ),
                    //                 Color::RED,
                    //             ));
                    //         }
                    //     }
                    //     1 => {
                    //         // Top

                    //         // Bottom left
                    //         line_points.push(Vertex::with_pos_color(
                    //             Vec2::new(x as f32 * grid_size, y as f32 * grid_size),
                    //             Color::RED,
                    //         ));
                    //         line_points.push(Vertex::with_pos_color(
                    //             Vec2::new(
                    //                 (x as f32 + 0.5) as f32 * grid_size,
                    //                 (y as f32 + 0.5) as f32 * grid_size,
                    //             ),
                    //             Color::RED,
                    //         ));

                    //         // Bottom right
                    //         line_points.push(Vertex::with_pos_color(
                    //             Vec2::new(
                    //                 (x as f32 + 1.0) as f32 * grid_size,
                    //                 y as f32 * grid_size,
                    //             ),
                    //             Color::RED,
                    //         ));
                    //         line_points.push(Vertex::with_pos_color(
                    //             Vec2::new(
                    //                 (x as f32 + 0.5) as f32 * grid_size,
                    //                 (y as f32 + 0.5) as f32 * grid_size,
                    //             ),
                    //             Color::RED,
                    //         ));

                    //         // Top edge
                    //         if y == 0 || json_data[y - 1][x] == 0 {
                    //             line_points.push(Vertex::with_pos_color(
                    //                 Vec2::new(x as f32 * grid_size, y as f32 * grid_size),
                    //                 Color::RED,
                    //             ));
                    //             line_points.push(Vertex::with_pos_color(
                    //                 Vec2::new((x + 1) as f32 * grid_size, y as f32 * grid_size),
                    //                 Color::RED,
                    //             ));
                    //         }
                    //     }
                    //     2 => {
                    //         // Left

                    //         // Top right
                    //         line_points.push(Vertex::with_pos_color(
                    //             Vec2::new(x as f32 * grid_size, y as f32 * grid_size),
                    //             Color::RED,
                    //         ));
                    //         line_points.push(Vertex::with_pos_color(
                    //             Vec2::new(
                    //                 (x as f32 + 0.5) as f32 * grid_size,
                    //                 (y as f32 + 0.5) as f32 * grid_size,
                    //             ),
                    //             Color::RED,
                    //         ));

                    //         // Bottom right
                    //         line_points.push(Vertex::with_pos_color(
                    //             Vec2::new(x as f32 * grid_size, (y + 1) as f32 * grid_size),
                    //             Color::RED,
                    //         ));
                    //         line_points.push(Vertex::with_pos_color(
                    //             Vec2::new(
                    //                 (x as f32 + 0.5) as f32 * grid_size,
                    //                 (y as f32 + 0.5) as f32 * grid_size,
                    //             ),
                    //             Color::RED,
                    //         ));

                    //         // Left edge
                    //         if x == 0 || json_data[y][x - 1] == 0 {
                    //             line_points.push(Vertex::with_pos_color(
                    //                 Vec2::new(x as f32 * grid_size, y as f32 * grid_size),
                    //                 Color::RED,
                    //             ));
                    //             line_points.push(Vertex::with_pos_color(
                    //                 Vec2::new(x as f32 * grid_size, (y + 1) as f32 * grid_size),
                    //                 Color::RED,
                    //             ));
                    //         }
                    //     }
                    //     3 => {
                    //         // Right

                    //         // Top left
                    //         line_points.push(Vertex::with_pos_color(
                    //             Vec2::new((x + 1) as f32 * grid_size, y as f32 * grid_size),
                    //             Color::RED,
                    //         ));
                    //         line_points.push(Vertex::with_pos_color(
                    //             Vec2::new(
                    //                 (x as f32 + 0.5) as f32 * grid_size,
                    //                 (y as f32 + 0.5) as f32 * grid_size,
                    //             ),
                    //             Color::RED,
                    //         ));

                    //         // Bottom left
                    //         line_points.push(Vertex::with_pos_color(
                    //             Vec2::new((x + 1) as f32 * grid_size, (y + 1) as f32 * grid_size),
                    //             Color::RED,
                    //         ));
                    //         line_points.push(Vertex::with_pos_color(
                    //             Vec2::new(
                    //                 (x as f32 + 0.5) as f32 * grid_size,
                    //                 (y as f32 + 0.5) as f32 * grid_size,
                    //             ),
                    //             Color::RED,
                    //         ));

                    //         // Right edge
                    //         if x == json_data[y].len() - 1 || json_data[y][x + 1] == 0 {
                    //             line_points.push(Vertex::with_pos_color(
                    //                 Vec2::new((x + 1) as f32 * grid_size, y as f32 * grid_size),
                    //                 Color::RED,
                    //             ));
                    //             line_points.push(Vertex::with_pos_color(
                    //                 Vec2::new(
                    //                     (x + 1) as f32 * grid_size,
                    //                     (y + 1) as f32 * grid_size,
                    //                 ),
                    //                 Color::RED,
                    //             ));
                    //         }
                    //     }
                    //     _ => {}
                    // }
                }
                _ => {}
            }
        }
    }

    return line_points;
}
