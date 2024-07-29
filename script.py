import os
import multiprocessing

A = ['diagonalwise', 'rowwise']
B = ['vc1', 'vc8', 'vc64']

def run(a, b): 
    os.system('./run.sh System_Workload/8_SN40L_GPT3_1.7B_2048/mesh/'+a+'/'+b+' 2_2 > System_Workload/8_SN40L_GPT3_1.7B_2048/mesh/'+a+'/'+b+'/2_2.txt')

programs = []
for a in A:
    for b in B:
        p = multiprocessing.Process(target=run, args=(a, b, ))
        programs.append(p)
        p.start()

for program in programs:
    program.join()
    