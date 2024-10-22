import os
import multiprocessing

os.system('./run.sh System_Workload/DLRM/8x8/ skip mesh 16 10240 > System_Workload/DLRM/8x8/mesh_10240.txt')
os.system('./run.sh System_Workload/DLRM/8x8/ skip torus 16 10240 > System_Workload/DLRM/8x8/torus_10240.txt')
os.system('./run.sh System_Workload/DLRM/8x8/ skip dragonfly 16 10240 > System_Workload/DLRM/8x8/dragonfly_10240.txt')

