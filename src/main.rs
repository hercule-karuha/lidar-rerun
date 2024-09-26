use dora_node_api::{DoraNode, Event};
use dotenv::dotenv;
use rerun::{default_flush_timeout, Points3D};
use std::env;
use std::{
    borrow::Borrow,
    net::{Ipv4Addr, SocketAddr},
    str::FromStr,
};

fn main() {
    // 加载 .env 文件中的环境变量
    dotenv().ok();

    // 从环境变量获取 IP 地址
    let remote_ip = load_env_var("REMOTE_IP").expect("Environment variable REMOTE_IP not set");
    let ip_address = Ipv4Addr::from_str(&remote_ip).expect("Invalid IP address");
    let addr = SocketAddr::from((ip_address, 9876));

    // 从环境变量获取 POINT_CLOUD_ID 和 PATH_ID
    let point_cloud_id =
        load_env_var("POINT_CLOUD_ID").expect("Environment variable POINT_CLOUD_ID not set");
    let path_id = load_env_var("PATH_ID").expect("Environment variable PATH_ID not set");

    let (mut _node, mut events) = DoraNode::init_from_env().unwrap();

    let rec = rerun::RecordingStreamBuilder::new("lidar point cloud and track")
        .connect_opts(addr, default_flush_timeout())
        .unwrap();
    rec.set_time_seconds("stable_time", 0f64);

    loop {
        let event = match events.recv() {
            Some(input) => input,
            None => break,
        };

        match event {
            Event::Input {
                id,
                metadata: _metadata,
                data,
            } => match id.as_str() {
                id if id == point_cloud_id => {
                    println!("receive point cloud !!!");

                    let raw_data: Vec<u8> = data.borrow().try_into().unwrap();

                    // 提取时间戳信息
                    // let seq = u32::from_le_bytes(raw_data[0..4].try_into().unwrap());
                    let stamp = i64::from_le_bytes(raw_data[4..12].try_into().unwrap());

                    rec.set_time_nanos("stable_time", stamp);

                    // 提取点云数据的 x, y, z 坐标
                    let mut points = Vec::new();
                    // let colors = Vec::new();
                    let point_data = &raw_data[16..];

                    for chunk in point_data.chunks_exact(16) {
                        let x = f32::from_le_bytes(chunk[0..4].try_into().unwrap());
                        let y = f32::from_le_bytes(chunk[4..8].try_into().unwrap());
                        let z = f32::from_le_bytes(chunk[8..12].try_into().unwrap());
                        // let i = f32::from_le_bytes(chunk[12..16].try_into().unwrap());
                        // let color = rerun::Color::f
                        points.push([x, y, z]);
                    }
                    let pcl: rerun::Points3D = Points3D::new(points);
                    rec.log("lidar/pcl", &pcl).unwrap();
                }
                id if id == path_id => {
                    let raw_data: Vec<u8> = data.borrow().try_into().unwrap();

                    // 解析轨迹数据
                    let mut x_coords = Vec::new();
                    let mut y_coords = Vec::new();

                    //code here
                    let half_len = raw_data.len() / 2;

                    // 提取所有 x 坐标
                    for chunk in raw_data[..half_len].chunks_exact(4) {
                        let x = f32::from_le_bytes(chunk.try_into().unwrap());
                        x_coords.push(x);
                    }

                    // 提取所有 y 坐标
                    for chunk in raw_data[half_len..].chunks_exact(4) {
                        let y = f32::from_le_bytes(chunk.try_into().unwrap());
                        y_coords.push(y);
                    }

                    // 确保 x 和 y 坐标的数量匹配
                    assert_eq!(x_coords.len(), y_coords.len());

                    // 创建线条数据
                    let mut strips = Vec::new();
                    let mut current_strip = Vec::new();

                    for i in 0..x_coords.len() {
                        current_strip.push([x_coords[i], y_coords[i]]);
                    }

                    strips.push(current_strip);

                    // 记录线条数据
                    let line_strips = rerun::LineStrips2D::new(strips).with_colors([0xFF0000FF]);

                    rec.log("lidar/path", &line_strips).unwrap();
                }
                other => eprintln!("Ignoring unexpected input `{other}`"),
            },
            Event::Stop => println!("Received manual stop"),
            other => eprintln!("Received unexpected input: {other:?}"),
        }
    }
}

fn load_env_var(key: &str) -> Result<String, env::VarError> {
    env::var(key)
}
