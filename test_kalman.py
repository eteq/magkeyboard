import re

import numpy as np

class KalmanFilter2D:
    def __init__(self, dt, process_noise, drift_process_noise, measurement_noise):
        # dt: Time step (usually 1 if points are equidistant)
        self.dt = dt
        
        # State vector [value, drift]
        self.x = np.array([[0], [0]]) 
        self.x = np.array([[2200], [2e-3]])  # an initial guess
        
        # State Transition Matrix (Predicts next state)
        self.F = np.array([[1, self.dt],
                           [0, 1]])
        
        # Measurement Matrix (We only measure the 'value', not the 'drift')
        self.H = np.array([[1, 0]])
        
        # Covariance Matrices
        self.P = np.eye(2) * 100  # Initial uncertainty
        self.P = np.array([[9.55515759e-01, 4.38029175e-08],
       [9.46599013e-04, 4.57917597e-09]]) # an initial guess from running for a while

        #self.Q = np.eye(2) * process_noise
        self.Q = np.array([
            [process_noise, 0],    # Value can change somewhat quickly
            [drift_process_noise, 0] # Drift is almost constant
        ])
        self.R = np.array([[measurement_noise]])

    def update(self, z):
        # 1. Predict
        self.x = self.F @ self.x
        self.P = self.F @ self.P @ self.F.T + self.Q
        
        # 2. Update (Correct with measurement z)
        z = np.array([[z]])
        y = z - (self.H @ self.x) # Innovation (error)
        S = self.H @ self.P @ self.H.T + self.R
        K = self.P @ self.H.T @ np.linalg.inv(S) # Kalman Gain
        
        self.x = self.x + (K @ y)
        self.P = (np.eye(2) - (K @ self.H)) @ self.P
        
        return self.x[0, 0], self.x[1, 0] # Returns (filtered_value, estimated_drift)

# --- Usage Example ---
#kf = KalmanFilter2D(dt=1, process_noise=0.001, measurement_noise=0.5)

# When you get a new data point:
#filtered_val, current_drift = kf.update(10.5)


def load_data(fn):
    ts = []
    datalines = []
    with open(fn) as f:
        for line in f:
            cleanline = line.split('(maghand_firmware')[0].strip()
            hzline = re.match(r'.*running sampler continuous @ (\d.*) kHz.*', cleanline)
            if hzline:
                freq = float(hzline.group(1)) * 1000    
                continue
            timeline = re.match(r'.*Sample set (\d.*) took us=(\d.*)', cleanline)
            if timeline:
                ts.append(int(timeline.group(2)))
                continue

            if 'bufs:' in cleanline:
                data_str = cleanline.split('bufs:')[1]
                datalines.append(eval(data_str))
    datarr = np.array(datalines).squeeze()
    datarr = datarr.reshape(np.prod(datarr.shape)//datarr.shape[-1], datarr.shape[-1])

    t1d = np.array(ts)

    ts = t1d.reshape(t1d.size, 1)*1e-6 + np.arange(datarr.shape[-1])/freq
    return ts, datarr, freq


# In [267]: datai=34;kf=KalmanFilter2D(1, process_noise=.01, drift_process_noise=.000001, measurement_
#         ⋮ noise=10);kfdat, kfdrift = np.array([kf.update(d) for d in data[datai]]).T;plt.clf();plt.p
#         ⋮ lot(ts[datai],data[datai]);plt.plot(ts[datai], kfdat);plt.twinx();plt.plot(ts[datai],kfdri
#         ⋮ ft,'C4')
