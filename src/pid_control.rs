use std::ops::Range;

pub struct Pid32 {
    /// proportional gain.
    kp: f32,
    /// integration gain.
    ki: f32,
    /// derivative gain.
    kd: f32,
    /// low pass filter time constant.
    tau: f32,

    lim_min: f32,
    lim_max: f32,

    lim_min_int: f32,
    lim_max_int: f32,

    delta_time: f32,

    integrator: f32,
    prev_error: f32,
    differentiator: f32,
    prev_measurement: f32,

    target: f32,
    prev_output: f32,
}

impl Pid32 {
    pub fn new(kp: f32, ki: f32, kd: f32, tau: f32) -> Self {
        Self {
            kp,
            ki,
            kd,
            tau,
            lim_min: 0.0,
            lim_max: 0.0,
            lim_min_int: 0.0,
            lim_max_int: 0.0,
            delta_time: 0.0,
            integrator: 0.0,
            prev_error: 0.0,
            differentiator: 0.0,
            prev_measurement: 0.0,
            target: 0.0,
            prev_output: 0.0,
        }
    }

    pub fn set_limits(&mut self, output: Range<f32>, integral: Range<f32>) {
        self.lim_min = output.start;
        self.lim_max = output.end;
        self.lim_min_int = integral.start;
        self.lim_max_int = integral.end;
    }

    pub fn update(&mut self, set_point: f32, measurement: f32) -> f32 {
        let error = set_point - measurement;
        let proportional = self.kp * error;

        self.integrator = f32::clamp(
            self.integrator + (0.5 * self.ki * self.delta_time * (error + self.prev_error)),
            self.lim_min_int,
            self.lim_max_int,
        );

        self.differentiator = -(2.0 * self.kd * (measurement - self.prev_measurement)
            + (2.0 * self.tau - self.delta_time) * self.differentiator)
            / (2.0 * self.tau + self.delta_time);

        self.prev_error = error;
        self.prev_measurement = measurement;

        f32::clamp(
            proportional + self.integrator + self.differentiator,
            self.lim_min,
            self.lim_max,
        )
    }
}
