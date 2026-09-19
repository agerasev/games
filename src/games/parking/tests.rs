use super::*;
fn step(round: &mut Round, control: Control, count: usize) {
    for _ in 0..count {
        round.advance(control, STEP);
    }
}
#[test]
fn forward_reverse_steering_and_brakes_follow_car_motion() {
    for sign in [-1.0, 1.0] {
        let mut car = Car::new(Vec2::ZERO, 0.0);
        for _ in 0..120 {
            car.step(Control {
                drive: sign,
                steer: 1.0,
                brake: false,
            });
        }
        assert!(car.pos.x * sign > 0.5);
        assert!(
            *car.angle * sign > 0.1,
            "reverse changes the turn direction"
        );
        for _ in 0..120 {
            car.step(Control {
                brake: true,
                ..Default::default()
            });
        }
        assert_eq!(car.speed, 0.0);
        let angle = *car.angle;
        for _ in 0..120 {
            car.step(Control {
                steer: -1.0,
                ..Default::default()
            });
        }
        assert_eq!(*car.angle, angle, "steering at rest cannot rotate the car");
    }
}
#[test]
fn rotated_boxes_test_full_body_and_separating_axes() {
    let a = Box2::car(Vec2::ZERO, 0.5);
    assert!(a.overlaps(Box2::car(Vec2::new(2.0, 0.0), -0.5)));
    assert!(!a.overlaps(Box2::car(Vec2::new(0.0, 5.0), 0.5)));
    let bay = Box2 {
        center: Vec2::ZERO,
        half: Vec2::new(3.0, 1.2),
        angle: 0.0,
    };
    assert!(bay.contains(Box2::car(Vec2::ZERO, 0.0)));
    assert!(!bay.contains(Box2::car(Vec2::new(1.0, 0.0), 0.0)));
    assert!(!bay.contains(Box2::car(Vec2::ZERO, 0.3)));
}
#[test]
fn parking_requires_full_containment_alignment_and_a_stopped_dwell() {
    let mut r = Round::new(2);
    *r.car.pos = r.level.target.center;
    *r.car.angle = std::f32::consts::PI;
    step(&mut r, Control::default(), 60);
    assert_eq!(r.status, Status::Driving);
    assert!(r.hold > 0.4);
    r.car.speed = 0.5;
    r.advance(Control::default(), STEP);
    assert_eq!(r.hold, 0.0);
    r.car.speed = 0.0;
    step(&mut r, Control::default(), 120);
    assert_eq!(r.status, Status::Parked);
    let pos = *r.car.pos;
    step(
        &mut r,
        Control {
            drive: 1.0,
            ..Default::default()
        },
        120,
    );
    assert_eq!(*r.car.pos, pos);
    assert!(!r.needs_update(Control::default()));
}
#[test]
fn collisions_stop_the_round_and_traffic_keeps_its_lane() {
    for reason in [Crash::Curb, Crash::ParkedCar, Crash::Traffic] {
        let mut r = Round::new(3);
        *r.car.pos = match reason {
            Crash::Curb => Vec2::new(1.0, 12.0),
            Crash::ParkedCar => r.level.parked[0].center,
            Crash::Traffic => r.level.traffic[0].body.center,
        };
        r.advance(Control::default(), STEP);
        assert_eq!(r.status, Status::Crashed(reason));
    }
    let mut t = Traffic {
        body: Box2::car(Vec2::new(44.99, 12.0), 0.0),
        velocity: 3.0,
    };
    t.step();
    assert!(t.body.center.x < -4.9 && t.body.center.y == 12.0);
}
#[test]
fn fixed_steps_match_across_render_rates_and_levels_start_clear() {
    for index in 0..NAMES.len() {
        let mut r = Round::new(index);
        r.advance(Control::default(), STEP);
        assert_eq!(r.status, Status::Driving);
        assert!(!r.inside());
    }
    let mut a = Round::new(1);
    let mut b = Round::new(1);
    let control = Control {
        drive: 1.0,
        steer: 0.3,
        brake: false,
    };
    for _ in 0..60 {
        a.advance(control, 1.0 / 60.0);
    }
    for _ in 0..30 {
        b.advance(control, 1.0 / 30.0);
    }
    assert!((*a.car.pos - *b.car.pos).length() < 1e-5);
    assert!((*a.car.angle - *b.car.angle).abs() < 1e-5);
}
#[test]
fn first_space_can_be_reached_by_driving_and_braking() {
    let mut r = Round::new(0);
    for _ in 0..2400 {
        let remaining = r.level.target.center.y - r.car.pos.y;
        let brake = r.car.speed * r.car.speed / 16.0 + 0.1 >= remaining;
        r.advance(
            Control {
                drive: if remaining > 0.15 && !brake { 1.0 } else { 0.0 },
                brake,
                steer: 0.0,
            },
            STEP,
        );
        if r.status != Status::Driving {
            break;
        }
    }
    assert_eq!(r.status, Status::Parked);
}
