use super::*;

fn tick(flight: &mut Flight, control: Control, count: usize) {
    for _ in 0..count {
        flight.advance(control, STEP);
    }
}

#[test]
fn lunar_free_fall_and_thrust_match_the_equations() {
    let mut fall = Flight::new(0);
    let initial = *fall.craft.pos;
    tick(&mut fall, Control::default(), 120);
    assert!((fall.craft.pos.y - (initial.y - 0.5 * GRAVITY)).abs() < 0.002);
    assert!((fall.craft.vel.y + GRAVITY).abs() < 0.002);
    assert_eq!(fall.fuel, 100.0);
    let mut burn = Flight::new(0);
    tick(
        &mut burn,
        Control {
            thrust: true,
            turn: 0.0,
        },
        120,
    );
    assert!((burn.craft.vel.y - (5.4 - GRAVITY)).abs() < 0.002);
    assert!((burn.fuel - 96.0).abs() < 0.002);
    assert!(!burn.particles.is_empty());
}

#[test]
fn rotation_redirects_thrust_and_empty_fuel_cannot_power_the_engine() {
    let mut flight = Flight::new(0);
    tick(
        &mut flight,
        Control {
            thrust: true,
            turn: 1.0,
        },
        120,
    );
    assert!(*flight.craft.angle > 0.0 && flight.craft.vel.x < -0.1);
    flight.fuel = 0.01;
    flight.advance(
        Control {
            thrust: true,
            turn: 0.0,
        },
        STEP,
    );
    assert_eq!(flight.fuel, 0.0);
    let vy = flight.craft.vel.y;
    flight.advance(
        Control {
            thrust: true,
            turn: 0.0,
        },
        STEP,
    );
    assert_eq!(flight.power, 0.0);
    assert!((flight.craft.vel.y - vy + GRAVITY * STEP).abs() < 1e-5);
}

#[test]
fn touchdown_requires_both_feet_on_pad_and_safe_motion() {
    for (x_offset, speed, angle, spin, expected) in [
        (0.0, Vec2::new(0.2, -1.0), 0.0, 0.0, Status::Landed),
        (
            0.0,
            Vec2::new(0.0, -4.0),
            0.0,
            0.0,
            Status::Crashed(Crash::Speed),
        ),
        (
            0.0,
            Vec2::new(2.0, -1.0),
            0.0,
            0.0,
            Status::Crashed(Crash::Speed),
        ),
        (
            0.0,
            Vec2::new(0.0, -1.0),
            0.4,
            0.0,
            Status::Crashed(Crash::Tilt),
        ),
        (
            0.0,
            Vec2::new(0.0, -1.0),
            0.0,
            1.0,
            Status::Crashed(Crash::Tilt),
        ),
        (
            15.0,
            Vec2::new(0.0, -1.0),
            0.0,
            0.0,
            Status::Crashed(Crash::Terrain),
        ),
    ] {
        let mut f = Flight::new(0);
        *f.craft.pos = Vec2::new(
            f.terrain.pad.center() + x_offset,
            f.terrain.pad.height + FOOT_DROP,
        );
        *f.craft.vel = speed;
        *f.craft.angle = angle;
        *f.craft.spin = spin;
        f.advance(Control::default(), STEP);
        assert_eq!(f.status, expected);
        let pos = *f.craft.pos;
        let fuel = f.fuel;
        tick(
            &mut f,
            Control {
                thrust: true,
                turn: 1.0,
            },
            120,
        );
        assert_eq!(*f.craft.pos, pos);
        assert_eq!(f.fuel, fuel);
    }
}

#[test]
fn fixed_steps_are_independent_of_render_cadence_and_particles_are_bounded() {
    let mut a = Flight::new(0);
    let mut b = Flight::new(0);
    let input = Control {
        thrust: true,
        turn: 0.3,
    };
    for _ in 0..60 {
        a.advance(input, 1.0 / 60.0);
    }
    for _ in 0..30 {
        b.advance(input, 1.0 / 30.0);
    }
    assert!((*a.craft.pos - *b.craft.pos).length() < 1e-4);
    assert!((a.fuel - b.fuel).abs() < 1e-5);
    assert_eq!(a.particles.len(), b.particles.len());
    for _ in 0..5000 {
        a.advance(input, STEP);
    }
    assert!(a.particles.len() <= MAX_PARTICLES);
    assert!(a.particles.iter().all(|p| p.pos.is_finite()));
    assert!(a.craft.pos.is_finite());
}

#[test]
fn first_site_can_be_landed_and_no_thrust_eventually_crashes() {
    let mut assisted = Flight::new(0);
    let mut fall = Flight::new(0);
    for _ in 0..3600 {
        let target_speed = -(assisted.altitude() * 0.5).clamp(0.8, 5.0);
        assisted.advance(
            Control {
                thrust: assisted.craft.vel.y < target_speed,
                turn: 0.0,
            },
            STEP,
        );
        fall.advance(Control::default(), STEP);
    }
    assert_eq!(assisted.status, Status::Landed);
    assert!(assisted.fuel > 0.0);
    assert_eq!(fall.status, Status::Crashed(Crash::Speed));
}

#[test]
fn terrain_ridges_and_play_area_bounds_are_solid() {
    let terrain = Terrain {
        pad: Terrain::new(0).pad,
        points: vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(60.0, 8.0),
            Vec2::new(120.0, 0.0),
        ],
    };
    let mut flight = Flight::new(0);
    *flight.craft.pos = Vec2::new(60.0, 10.3);
    assert!(terrain.contact(&flight.craft.hull()));
    *flight.craft.pos = Vec2::new(-1.0, 64.0);
    flight.advance(Control::default(), STEP);
    assert_eq!(flight.status, Status::Crashed(Crash::Bounds));
}

#[test]
fn exhaust_hits_the_surface_and_impact_telemetry_is_preserved() {
    let mut f = Flight::new(0);
    f.craft.pos.y = 14.0;
    tick(
        &mut f,
        Control {
            thrust: true,
            turn: 0.0,
        },
        120,
    );
    assert!(f.particles.iter().any(|p| p.dust));
    assert!(
        f.particles
            .iter()
            .all(|p| p.pos.y >= f.terrain.height(p.pos.x))
    );
    *f.craft.pos = Vec2::new(60.0, 8.4);
    *f.craft.vel = Vec2::new(0.0, -8.0);
    f.advance(Control::default(), STEP);
    assert_eq!(f.status, Status::Crashed(Crash::Speed));
    assert!(f.craft.vel.y < -8.0);
}
