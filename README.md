# 使用rerun可视化激光雷达的点云与轨迹数据的dora节点


## 安装

在用于可视化的远程主机上安装rerun viewer：
`cargo install rerun-cli --locked`


使用.env文件配置IP地址以及输入数据的ID
.env文件示例：
```
REMOTE_IP=127.0.0.1
POINT_CLOUD_ID=pointcloud
PATH_ID=raw_path
```

在yaml文件中配置节点的输入与输出
示例：
```
  - id: lidar-rerun
    custom:
      source: lidar-rerun/target/debug/lidar-rerun
      inputs:
          pointcloud: rslidar_driver/pointcloud
          raw_path: path_input/raw_path
```

在用于可视化的远程主机中的终端运行
```
rerun
```
来接受需要可视化的数据

然后运行dora数据流程序向远程主机中传输数据