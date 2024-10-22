import os
import multiprocessing

workload = ['16x16_dragonfly']
topology = [['skip', 'dragonfly']]

def run(work, on, off):
    os.system('./run.sh System_Workload/LLM/'+work+'/ '+on+' '+off+' 16 1024 > System_Workload/LLM/'+work+'/'+off+'.txt')

programs = []
for work in workload:
    for on, off in topology:
        run(work, on, off)