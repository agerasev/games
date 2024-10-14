use macroquad::math::Vec2;

pub trait Parameter {
    type Derivative: Default;
    fn step(&mut self, dp: &Self::Derivative, dt: f32);
}

pub trait Wrapper<S: Solver<Wrapper<P> = Self> + ?Sized, P: Parameter>: Sized {
    fn wrap(p: P) -> Self;

    fn p(&self) -> &P;
    fn p_mut(&mut self) -> &mut P;

    fn dp(&self) -> &P::Derivative;
    fn dp_mut(&mut self) -> &mut P::Derivative;
}

pub trait Solver {
    type Wrapper<P: Parameter>: Wrapper<Self, P>;
    fn solve<S: System<Self>>(&self, system: &mut S, dt: f32);
}

pub trait Visitor {
    type Solver: Solver + ?Sized;
    fn apply<P: Parameter>(&mut self, wp: &mut <Self::Solver as Solver>::Wrapper<P>);
}

pub trait System<S: Solver + ?Sized> {
    fn compute_derivatives(&mut self);
    fn visit_parameters<V: Visitor<Solver = S>>(&mut self, visitor: &mut V);
}

impl Parameter for Vec2 {
    type Derivative = Vec2;
    fn step(&mut self, dp: &Vec2, dt: f32) {
        *self += *dp * dt;
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, Debug)]
pub struct SecondOrder<P: Parameter> {
    pub p: P,
    pub dp: P::Derivative,
}

impl<P: Parameter<Derivative: Parameter>> Parameter for SecondOrder<P> {
    type Derivative = <P::Derivative as Parameter>::Derivative;
    fn step(&mut self, d2p: &Self::Derivative, dt: f32) {
        self.p.step(&self.dp, dt);
        self.dp.step(d2p, dt);
    }
}

pub struct Euler;

impl<P: Parameter> Wrapper<Euler, P> for SecondOrder<P> {
    fn wrap(p: P) -> Self {
        Self {
            p,
            dp: P::Derivative::default(),
        }
    }

    fn p(&self) -> &P {
        &self.p
    }
    fn p_mut(&mut self) -> &mut P {
        &mut self.p
    }

    fn dp(&self) -> &P::Derivative {
        &self.dp
    }
    fn dp_mut(&mut self) -> &mut P::Derivative {
        &mut self.dp
    }
}

struct EulerStep {
    dt: f32,
}

impl Visitor for EulerStep {
    type Solver = Euler;
    fn apply<P: Parameter>(&mut self, wp: &mut <Self::Solver as Solver>::Wrapper<P>) {
        wp.p.step(&wp.dp, self.dt);
    }
}

impl Solver for Euler {
    type Wrapper<P: Parameter> = SecondOrder<P>;

    fn solve<S: System<Self>>(&self, system: &mut S, dt: f32) {
        system.compute_derivatives();
        system.visit_parameters(&mut EulerStep { dt });
    }
}
