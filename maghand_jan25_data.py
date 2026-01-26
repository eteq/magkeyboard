# coding: utf-8
get_ipython().run_line_magic('matplotlib', '')
from matplotlib import pyplot as plt
import numpy as np
ls=[l.split('(maghand_firmware')[0].strip() for l in open('magtest').read().split('\n') if '[INFO ] key values' in l];ts = np.array([float(li.split(' ')[0]) for li in ls]);vs = np.array([eval(li.split(' : ')[1]) for li in ls]);nvs = np.array([eval(li.split(' : ')[2]) for li in ls]);mxs = np.array([eval(li.split(' : ')[3]) for li in ls]);mis = np.array([eval(li.split(' : ')[4]) for li in ls]);
pl
plt.clf();plt.subplot(2,1,1);keyidxs=[6,14];plt.plot(ts, vs[:, keyidxs]);plt.plot(ts, mis[:, keyidxs],c='k', ls='-');plt.plot(ts, mxs[:, keyidxs],c='k', ls=':');plt.subplot(2,1,2,sharex=plt.gca());plt.plot(ts,nvs[:, keyidxs])
424
424
25*6
25*6*4
25*6*4
25*6*4
(25*6*4)
(25*6*4)
80000
1/80000
(1/80000)*25*6*4
12.5*6*4
12.5*6*4*25
12847
12847/25
(12847/25)/.01
((12847/25)/.01)/1000
ls=[l.split('(maghand_firmware')[0].strip() for l in open('magtest').read().split('\n') if '[INFO ] key values' in l];ts = np.array([float(li.split(' ')[0]) for li in ls]);vs = np.array([eval(li.split(' : ')[1]) for li in ls]);nvs = np.array([eval(li.split(' : ')[2]) for li in ls]);mxs = np.array([eval(li.split(' : ')[3]) for li in ls]);mis = np.array([eval(li.split(' : ')[4]) for li in ls]);
plt.clf();plt.subplot(2,1,1);keyidxs=[6,14];plt.plot(ts, vs[:, keyidxs]);plt.plot(ts, mis[:, keyidxs],c='k', ls='-');plt.plot(ts, mxs[:, keyidxs],c='k', ls=':');plt.subplot(2,1,2,sharex=plt.gca());plt.plot(ts,nvs[:, keyidxs])
plt.clf();plt.subplot(2,1,1);keyidxs=list(range(24));plt.plot(ts, vs[:, keyidxs]);plt.plot(ts, mis[:, keyidxs],c='k', ls='-');plt.plot(ts, mxs[:, keyidxs],c='k', ls=':');plt.subplot(2,1,2,sharex=plt.gca());plt.plot(ts,nvs[:, keyidxs])
plt.clf();plt.subplot(2,1,1);keyidxs=[6,14,20];plt.plot(ts, vs[:, keyidxs]);plt.plot(ts, mis[:, keyidxs],c='k', ls='-');plt.plot(ts, mxs[:, keyidxs],c='k', ls=':');plt.subplot(2,1,2,sharex=plt.gca());plt.plot(ts,nvs[:, keyidxs])
plt.clf();plt.subplot(2,1,1);keyidxs=[6,14,21];plt.plot(ts, vs[:, keyidxs]);plt.plot(ts, mis[:, keyidxs],c='k', ls='-');plt.plot(ts, mxs[:, keyidxs],c='k', ls=':');plt.subplot(2,1,2,sharex=plt.gca());plt.plot(ts,nvs[:, keyidxs])
plt.clf();plt.subplot(2,1,1);keyidxs=[6,14,22];plt.plot(ts, vs[:, keyidxs]);plt.plot(ts, mis[:, keyidxs],c='k', ls='-');plt.plot(ts, mxs[:, keyidxs],c='k', ls=':');plt.subplot(2,1,2,sharex=plt.gca());plt.plot(ts,nvs[:, keyidxs])
plt.clf();plt.subplot(2,1,1);keyidxs=[6,14,18, 22, 24];plt.plot(ts, vs[:, keyidxs]);plt.plot(ts, mis[:, keyidxs],c='k', ls='-');plt.plot(ts, mxs[:, keyidxs],c='k', ls=':');plt.subplot(2,1,2,sharex=plt.gca());plt.plot(ts,nvs[:, keyidxs])
plt.clf();plt.subplot(2,1,1);keyidxs=[6,14,18, 22, 21];plt.plot(ts, vs[:, keyidxs]);plt.plot(ts, mis[:, keyidxs],c='k', ls='-');plt.plot(ts, mxs[:, keyidxs],c='k', ls=':');plt.subplot(2,1,2,sharex=plt.gca());plt.plot(ts,nvs[:, keyidxs])
plt.clf();plt.subplot(2,1,1);keyidxs=[6,14,18, 22, 21];plt.plot(ts, vs[:, keyidxs]);plt.plot(ts, mis[:, keyidxs],c='k', ls='-');plt.plot(ts, mxs[:, keyidxs],c='k', ls=':');plt.subplot(2,1,2,sharex=plt.gca());plt.plot(ts,nvs[:, keyidxs])
[f'C{i}' for i in range(len(keyidxs))]
plt.clf();plt.subplot(2,1,1);keyidxs=[6,14,18, 22, 24];plt.plot(ts, vs[:, keyidxs]);plt.plot(ts, mis[:, keyidxs],c=[f'C{i}' for i in range(len(keyidxs))], ls='-');plt.plot(ts, mxs[:, keyidxs],c=[f'C{i}' for i in range(len(keyidxs))], ls=':');plt.subplot(2,1,2,sharex=plt.gca());plt.plot(ts,nvs[:, keyidxs])
plt.clf();plt.subplot(2,1,1);keyidxs=[6,14,18, 22, 23];plt.plot(ts, vs[:, keyidxs]);plt.plot(ts, mis[:, keyidxs],c=[f'C{i}' for i in range(len(keyidxs))], ls='-');plt.plot(ts, mxs[:, keyidxs],c=[f'C{i}' for i in range(len(keyidxs))], ls=':');plt.subplot(2,1,2,sharex=plt.gca());plt.plot(ts,nvs[:, keyidxs])
plt.clf();plt.subplot(2,1,1);keyidxs=[6,14,18, 22, 24];plt.plot(ts, vs[:, keyidxs]);plt.plot(ts, mis[:, keyidxs],c=[f'C{i}' for i in range(len(keyidxs))], ls='-');plt.plot(ts, mxs[:, keyidxs],cs=[f'C{i}' for i in range(len(keyidxs))], ls=':');plt.subplot(2,1,2,sharex=plt.gca());plt.plot(ts,nvs[:, keyidxs])
plt.clf();plt.subplot(2,1,1);keyidxs=[6,14,18, 22, 23];plt.plot(ts, vs[:, keyidxs]);plt.plot(ts, mis[:, keyidxs],c=[f'C{i}' for i in range(len(keyidxs))], ls='-');plt.plot(ts, mxs[:, keyidxs],cs=[f'C{i}' for i in range(len(keyidxs))], ls=':');plt.subplot(2,1,2,sharex=plt.gca());plt.plot(ts,nvs[:, keyidxs])
plt.clf();plt.subplot(2,1,1);keyidxs=[6,14,18, 22, 23];plt.plot(ts, vs[:, keyidxs]);plt.plot(ts, mis[:, keyidxs],cs=[f'C{i}' for i in range(len(keyidxs))], ls='-');plt.plot(ts, mxs[:, keyidxs],cs=[f'C{i}' for i in range(len(keyidxs))], ls=':');plt.subplot(2,1,2,sharex=plt.gca());plt.plot(ts,nvs[:, keyidxs])
plt.clf();plt.subplot(2,1,1);keyidxs=[6,14,18, 22, 23];plt.plot(ts, vs[:, keyidxs]);plt.plot(ts, mis[:, keyidxs],c=[f'C{i}' for i in range(len(keyidxs))], ls='-');plt.plot(ts, mxs[:, keyidxs],c=[f'C{i}' for i in range(len(keyidxs))], ls=':');plt.subplot(2,1,2,sharex=plt.gca());plt.plot(ts,nvs[:, keyidxs])
plt.clf();plt.subplot(2,1,1);keyidxs=[6,14,18, 22, 23];plt.plot(ts, vs[:, keyidxs]);plt.plot(ts, mis[:, keyidxs],c=[f'C{i}' for i in range(len(keyidxs))], ls='-');plt.plot(ts, mxs[:, keyidxs],c=[f'C{i}' for i in range(len(keyidxs))], ls=':');plt.subplot(2,1,2,sharex=plt.gca());plt.plot(ts,nvs[:, keyidxs])
plt.clf();plt.subplot(2,1,1);keyidxs=[6,14,18, 22, 23];plt.plot(ts, vs[:, keyidxs]);plt.plot(ts, mis[:, keyidxs],c=[f'C{i}' for i in range(len(keyidxs))], ls='-');plt.plot(ts, mxs[:, keyidxs],c=[f'C{i}' for i in range(len(keyidxs))], ls=':');plt.subplot(2,1,2,sharex=plt.gca());plt.plot(ts,nvs[:, keyidxs])
plt.clf();plt.subplot(2,1,1);keyidxs=[6,14,18, 22, 23];plt.plot(ts, vs[:, keyidxs]);plt.plot(ts, mis[:, keyidxs],c=[f'C{i}' for i in range(len(keyidxs))], ls='-');plt.plot(ts, mxs[:, keyidxs],c=[f'C{i}' for i in range(len(keyidxs))], ls=':');plt.subplot(2,1,2,sharex=plt.gca());plt.plot(ts,nvs[:, keyidxs])
plt.plot(ts, mis[:, keyidxs],c=[f'C{i}' for i in range(len(keyidxs))])
plt.plot(ts, mis[:, keyidxs],c=[f'C{i}' for i in range(len(keyidxs))])
len([f'C{i}' for i in range(len(keyidxs))])
mis[:, keyidxs].shape
get_ipython().run_line_magic('pinfo', 'plt.plot')
plt.clf();plt.subplot(2,1,1);keyidxs=[6,14,18, 22, 23];plt.plot(ts, vs[:, keyidxs]);plt.plot(ts, mis[:, keyidxs],c=[f'C{i}' for i in range(len(keyidxs))], ls='-');plt.plot(ts, mxs[:, keyidxs],c=[f'C{i}' for i in range(len(keyidxs))], ls=':');plt.subplot(2,1,2,sharex=plt.gca());plt.plot(ts,nvs[:, keyidxs])
plt.clf();plt.subplot(2,1,1);keyidxs=[6,14,18, 22, 21];plt.plot(ts, vs[:, keyidxs]);plt.plot(ts, mis[:, keyidxs],c='k', ls='-');plt.plot(ts, mxs[:, keyidxs],c='k', ls=':');plt.subplot(2,1,2,sharex=plt.gca());plt.plot(ts,nvs[:, keyidxs])
plt.clf();plt.subplot(2,1,1);keyidxs=[6,14,21];plt.plot(ts, vs[:, keyidxs]);plt.plot(ts, mis[:, keyidxs],c='k', ls='-');plt.plot(ts, mxs[:, keyidxs],c='k', ls=':');plt.subplot(2,1,2,sharex=plt.gca());plt.plot(ts,nvs[:, keyidxs])
ls=[l.split('(maghand_firmware')[0].strip() for l in open('magtest2').read().split('\n') if '[INFO ] key values' in l];ts = np.array([float(li.split(' ')[0]) for li in ls]);vs = np.array([eval(li.split(' : ')[1]) for li in ls]);nvs = np.array([eval(li.split(' : ')[2]) for li in ls]);mxs = np.array([eval(li.split(' : ')[3]) for li in ls]);mis = np.array([eval(li.split(' : ')[4]) for li in ls]);
plt.clf();plt.subplot(2,1,1);keyidxs=[6,14,18, 22, 21];plt.plot(ts, vs[:, keyidxs]);plt.plot(ts, mis[:, keyidxs],c='k', ls='-');plt.plot(ts, mxs[:, keyidxs],c='k', ls=':');plt.subplot(2,1,2,sharex=plt.gca());plt.plot(ts,nvs[:, keyidxs])
ls
ls=[l.split('(maghand_firmware')[0].strip() for l in open('magtest2').read().split('\n') if '[INFO ] key values' in l];ts = np.array([float(li.split(' ')[0]) for li in ls]);vs = np.array([eval(li.split(' : ')[1]) for li in ls]);nvs = np.array([eval(li.split(' : ')[2]) for li in ls]);mxs = np.array([eval(li.split(' : ')[3]) for li in ls]);mis = np.array([eval(li.split(' : ')[4]) for li in ls]);
ls
ls=[l.split('(maghand_firmware')[0].strip() for l in open('magtest2').read().split('\n') if '[INFO ] key values' in l];ts = np.array([float(li.split(' ')[0]) for li in ls]);vs = np.array([eval(li.split(' : ')[1]) for li in ls]);nvs = np.array([eval(li.split(' : ')[2]) for li in ls]);mxs = np.array([eval(li.split(' : ')[3]) for li in ls]);mis = np.array([eval(li.split(' : ')[4]) for li in ls]);
ls1 = [l.split('(maghand_firmware')[0].strip() for l in open('magtest2').read().split('\n')]
ls1
ls1 = [l.split('(maghand_firmware')[0].strip() for l in open('magtest2').read().split('\n')if 'key 0 adc' in l]
ls1
ls1 = [l.split('(maghand_firmware')[0].strip() for l in open('magtest2').read().split('\n')if 'key 0 adc' in l];fastt=np.array([float(li.split()[0]) for li in ls1]);fast=np.array([int(li.split()[-1])for li in ls1])
plt.figure()
ls1 = [l.split('(maghand_firmware')[0].strip() for l in open('magtest2').read().split('\n')if 'key 0 adc' in l];fastt=np.array([float(li.split()[0]) for li in ls1]);fast=np.array([int(li.split()[-1])for li in ls1])
ls1 = [l.split('(maghand_firmware')[0].strip() for l in open('magtest2').read().split('\n')if 'key 0 adc' in l];fastt=np.array([float(li.split()[0]) for li in ls1]);fast=np.array([int(li.split()[-1].split(',')[0])for li in ls1])
ls1 = [l.split('(maghand_firmware')[0].strip() for l in open('magtest2').read().split('\n')if 'key 0 adc' in l];fastt=np.array([float(li.split()[0]) for li in ls1]);fast=np.array([int(li.split()[-1].split(',')[0])for li in ls1]);fastv=np.array([float(li.split()[-1].split(',')[1])for li in ls1])
fastv
plt.
ls=[l.split('(maghand_firmware')[0].strip() for l in open('magtest2').read().split('\n') if '[INFO ] key values' in l];ts = np.array([float(li.split(' ')[0]) for li in ls]);vs = np.array([eval(li.split(' : ')[1]) for li in ls]);nvs = np.array([eval(li.split(' : ')[2]) for li in ls]);mxs = np.array([eval(li.split(' : ')[3]) for li in ls]);mis = np.array([eval(li.split(' : ')[4]) for li in ls]);
plt.clf();plt.subplot(2,1,1);keyidxs=[6,14,18, 22, 21];plt.plot(ts, vs[:, keyidxs]);plt.plot(ts, mis[:, keyidxs],c='k', ls='-');plt.plot(ts, mxs[:, keyidxs],c='k', ls=':');plt.subplot(2,1,2,sharex=plt.gca());plt.plot(ts,nvs[:, keyidxs])
plt.clf();plt.subplot(2,1,1);keyidxs=[0,6,14];plt.plot(ts, vs[:, keyidxs]);plt.plot(ts, mis[:, keyidxs],c='k', ls='-');plt.plot(ts, mxs[:, keyidxs],c='k', ls=':');plt.subplot(2,1,2,sharex=plt.gca());plt.plot(ts,nvs[:, keyidxs])
plt.clf();plt.subplot(2,1,1);keyidxs=[6,14,0];plt.plot(ts, vs[:, keyidxs]);plt.plot(ts, mis[:, keyidxs],c='k', ls='-');plt.plot(ts, mxs[:, keyidxs],c='k', ls=':');plt.subplot(2,1,2,sharex=plt.gca());plt.plot(ts,nvs[:, keyidxs])
plt.subplot(2,1,1);plt.plot(fastt, c='C3', ls='-.')
fastt
ts
fastt
plt.clf();plt.subplot(2,1,1);keyidxs=[6,14,0];plt.plot(ts, vs[:, keyidxs]);plt.plot(ts, mis[:, keyidxs],c='k', ls='-');plt.plot(ts, mxs[:, keyidxs],c='k', ls=':');plt.subplot(2,1,2,sharex=plt.gca());plt.plot(ts,nvs[:, keyidxs])
plt.subplot(2,1,1);plt.plot(fastt, fast, c='C3', ls='-.')
plt.subplot(2,1,1);plt.plot(fastt, fastv, c='C4', ls='-.')
plt.subplot(2,1,1);plt.plot(fastt, fast, c='C3', ls='.')
plt.subplot(2,1,1);plt.plot(fastt, fast,'.',c='C3')
plt.clf();plt.subplot(2,1,1);keyidxs=[6,14,0];plt.plot(ts, vs[:, keyidxs]);plt.plot(ts, mis[:, keyidxs],c='k', ls='-');plt.plot(ts, mxs[:, keyidxs],c='k', ls=':');plt.subplot(2,1,2,sharex=plt.gca());plt.plot(ts,nvs[:, keyidxs])
plt.subplot(2,1,1);plt.plot(fastt, fast,'.',c='C3');plt.plot(fastt, fastv,'-.',c='C3')
plt.subplot(2,1,1);plt.plot(fastt, fast,'.',c='C3');plt.plot(fastt, fastv,'-.',c='C4')
plt.subplot(2,1,1);plt.plot(fastt, fast,'.',c='C3');plt.plot(fastt, fastv,'-',c='C4')
ls=[l.split('(maghand_firmware')[0].strip() for l in open('magtest2').read().split('\n') if '[INFO ] key values' in l];ts = np.array([float(li.split(' ')[0]) for li in ls]);vs = np.array([eval(li.split(' : ')[1]) for li in ls]);nvs = np.array([eval(li.split(' : ')[2]) for li in ls]);mxs = np.array([eval(li.split(' : ')[3]) for li in ls]);mis = np.array([eval(li.split(' : ')[4]) for li in ls]);
ls1 = [l.split('(maghand_firmware')[0].strip() for l in open('magtest2').read().split('\n')if 'key 0 adc' in l];fastt=np.array([float(li.split()[0]) for li in ls1]);fast=np.array([int(li.split()[-1].split(',')[0])for li in ls1]);fastv=np.array([float(li.split()[-1].split(',')[1])for li in ls1])
plt.clf();plt.subplot(2,1,1);keyidxs=[6,14,0];plt.plot(ts, vs[:, keyidxs]);plt.plot(ts, mis[:, keyidxs],c='k', ls='-');plt.plot(ts, mxs[:, keyidxs],c='k', ls=':');plt.subplot(2,1,2,sharex=plt.gca());plt.plot(ts,nvs[:, keyidxs])
plt.subplot(2,1,1);plt.plot(fastt, fast,'.',c='C3');plt.plot(fastt, fastv,'-',c='C4')
fastt
ls1 = [l.split('(maghand_firmware')[0].strip() for l in open('magtest2').read().split('\n')if 'adc,val' in l];fastt=np.array([float(li.split()[0]) for li in ls1]);fast=np.array([int(li.split()[-1].split(',')[0])for li in ls1]);fastv=np.array([float(li.split()[-1].split(',')[1])for li in ls1])
ls1
plt.subplot(2,1,1);plt.plot(fastt, fast,'.',c='C3');plt.plot(fastt, fastv,'-',c='C4')
ls1 = [l.split('(maghand_firmware')[0].strip() for l in open('magtest2').read().split('\n')if 'adc,val' in l];fastt=np.array([float(li.split()[0]) for li in ls1]);fast=np.array([int(li.split()[-1].split(',')[0])for li in ls1]);fastv=np.array([float(li.split()[-1].split(',')[1])for li in ls1])
ls=[l.split('(maghand_firmware')[0].strip() for l in open('magtest2').read().split('\n') if '[INFO ] key values' in l];ts = np.array([float(li.split(' ')[0]) for li in ls]);vs = np.array([eval(li.split(' : ')[1]) for li in ls]);nvs = np.array([eval(li.split(' : ')[2]) for li in ls]);mxs = np.array([eval(li.split(' : ')[3]) for li in ls]);mis = np.array([eval(li.split(' : ')[4]) for li in ls]);
plt.clf();plt.subplot(2,1,1);keyidxs=[6,14,0];plt.plot(ts, vs[:, keyidxs]);plt.plot(ts, mis[:, keyidxs],c='k', ls='-');plt.plot(ts, mxs[:, keyidxs],c='k', ls=':');plt.subplot(2,1,2,sharex=plt.gca());plt.plot(ts,nvs[:, keyidxs])
plt.subplot(2,1,1);plt.plot(fastt, fast,'.',c='C3');plt.plot(fastt, fastv,'-',c='C4')
ls=[l.split('(maghand_firmware')[0].strip() for l in open('magtest2').read().split('\n') if '[INFO ] key values' in l];ts = np.array([float(li.split(' ')[0]) for li in ls]);vs = np.array([eval(li.split(' : ')[1]) for li in ls]);nvs = np.array([eval(li.split(' : ')[2]) for li in ls]);mxs = np.array([eval(li.split(' : ')[3]) for li in ls]);mis = np.array([eval(li.split(' : ')[4]) for li in ls]);
ls1 = [l.split('(maghand_firmware')[0].strip() for l in open('magtest2').read().split('\n')if 'adc,val' in l];fastt=np.array([float(li.split()[0]) for li in ls1]);fast=np.array([int(li.split()[-1].split(',')[0])for li in ls1]);fastv=np.array([float(li.split()[-1].split(',')[1])for li in ls1])
plt.subplot(2,1,1);plt.plot(fastt, fast,'.',c='C3');plt.plot(fastt, fastv,'-',c='C4')
plt.clf();plt.subplot(2,1,1);keyidxs=[6,14,0];plt.plot(ts, vs[:, keyidxs]);plt.plot(ts, mis[:, keyidxs],c='k', ls='-');plt.plot(ts, mxs[:, keyidxs],c='k', ls=':');plt.subplot(2,1,2,sharex=plt.gca());plt.plot(ts,nvs[:, keyidxs])
plt.subplot(2,1,1);plt.plot(fastt, fast,'.',c='C3');plt.plot(fastt, fastv,'-',c='C4')
get_ipython().run_line_magic('history', '')
get_ipython().run_line_magic('history', '--help')
get_ipython().run_line_magic('history', '--help')
