use bevy::{math::*, prelude::*, sprite::collide_aabb::*};
use rand::prelude::*;

//paddle
// position the paddle 60 units above the bottom wall - is the y-coordinate
// f32 is a 32-bit floating-point number
const PADDLE_START_Y: f32 = BOTTOM_WALL + 60.;
// Vec2 - representing 2D vector (120 units wide and 20 units tall)
const PADDLE_SIZE: Vec2 = Vec2::new(120.0, 20.0);
const PADDLE_COLOR: Color = Color::rgb(0.3, 0.3, 0.7);
// speed of the paddle - representing the number of units the paddle moves per second per frame
const PADDLE_SPEED: f32 = 500.0;

//ball
const BALL_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);
const BALL_STARTING_POSITION: Vec3 = Vec3::new(0.0, -50.0, 1.0);
const BALL_SIZE: Vec2 = Vec2::new(30.0, 30.0);
const BALL_SPEED: f32 = 400.0;
// ball move to the right and downward
const BALL_INITIAL_DIRECTION: Vec2 = Vec2::new(0.5, -0.5);

//wall
const LEFT_WALL: f32 = -450.;
const RIGHT_WALL: f32 = 450.;
const BOTTOM_WALL: f32 = -300.;
const TOP_WALL: f32 = 300.;

// 10 units thick
const WALL_THICKNESS: f32 = 10.0;
// total width the game area enclosed by left and right walls
const WALL_BLOCK_WIDTH: f32 = RIGHT_WALL - LEFT_WALL;
const WALL_BLOCK_HEIGHT: f32 = TOP_WALL - BOTTOM_WALL;
const WALL_COLOR: Color = Color::rgb(0.8, 0.8, 0.8);

//bricks
const BRICK_SIZE: Vec2 = Vec2::new(100., 30.);
const BRICK_COLOR: Color = Color::rgb(0.5, 0.5, 1.0);
const GAP_BETWEEN_PADDLE_AND_BRICKS: f32 = 270.0;
const GAP_BETWEEN_BRICKS: f32 = 5.0;
// vertical gap between the top row of the bricks and the ceiling(top boundary)
const GAP_BETWEEN_BRICKS_AND_CEILING: f32 = 20.0;
// horizontal gap between the bricks and the side boundaries (left and right walls)
const GAP_BETWEEN_BRICKS_AND_SIDES: f32 = 20.0;

//scoreboard
const SCOREBOARD_FONT_SIZE: f32 = 40.0;
// Px = pixels
const SCOREBOARD_TEXT_PADDING: Val = Val::Px(5.0);
const TEXT_COLOR: Color = Color::rgb(0.5, 0.5, 1.0);
const SCORE_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);

// ** Note **
// use .insert_resource when you want to add globally accesible data that can be shared and modified by multiple systems
// use .add_systems when u want to add logic that acts on entities and their components
// Game LifeCycle: Update, Startup, FixedUpdate
// Startup is use for initialization tasks that should happen once when the game starts e.g. create entities, load resources, initialize any state that tge game needs before the main loop begins (initialize score to 0)
// Update stage runs every frame. System in this stage are executed once per frame and are used for tasks that need to check or change regularly e.g., handling player input, moving characters, implementing game mechanics(shooting,...), check for conditions that change often (collision detection, health check)
// FixedUpdate stage  is used for updates that need to happen at a consistent rate, independent of the frame rate e.g., 

fn main() {
    App::new()
        // provide functionality like rendering, event handling, window management
        .add_plugins(DefaultPlugins)
        // add a resource to the application - a resource is a piece of data that can be accessed globally within the app
        .insert_resource(ClearColor(Color::rgb(0.9, 0.9, 0.9)))
        // adds a scoreboard resource to game with initial score 0 - the resource is globally accessible and can be used to track and display the player's score throughout the game
        .insert_resource(Scoreboard { score: 0 })
        // .add_systems is used to add systems. Systems are functions that run every frame and perform operations on entities and their components
        // run during the Update stage of the game loop
        // closes the game window when the escape key is pressed
        .add_systems(Update, (bevy::window::close_on_esc, update_scoreboard),)
        // runs once when the app starts
        .add_systems(Startup, setup)
        // used for physics updates and other operations that should occur at a fixed interval
        .add_systems(
            FixedUpdate,
            (
                move_paddle,
                apply_velocity,
                // ensures that collision checks happen after velocity have been applied
                check_ball_collisions.after(apply_velocity),
            ),
        )
        // game start and continuously runs, executing teh registered systems each frame until the game is closed
        .run();
}

#[derive(Component)]
struct Paddle;

#[derive(Component)]
// to access the size of a ball, use ball.size
struct Ball {
    size: Vec2,
}

// Deref and DerefMut are standard Rust traits. Deref allows a type to behave like a reference to another type, and DerefMut is its mutable counterpart.
// If you have a Velocity struct that primarily holds a Vec2, deriving Deref and DerefMut lets you use Velocity as if it were Vec2 directly.
#[derive(Component, Deref, DerefMut)]
// velocity = speed (magnitude) + direction
// Speed = Distance/Time
// velocity has both magnitude and direction
// spedd only has magnitude
// e.g., vector(3,4) means the object is moving 3 units in x-direction and 4 units in the y-direction each frame or second
// this is a tuple struct, to accessVec2 in Velocity, we need to used index-based access like velocity.0
struct Velocity(Vec2);

#[derive(Component)]
struct Collider {
    size: Vec2,
}

// Bundles are collections of components. 
// They make it easier to add multiple components to an entity at once.
#[derive(Bundle)]
// When you want to add a wall to your game, you use WallBundle. This automatically gives the wall entity both a sprite (visual representation) and a collider (physical boundary for collisions).
struct WallBundle {
    // This component deals with how the wall looks (its appearance, position, etc.).
    //  It typically includes components such as texture, transform
    sprite_bundle: SpriteBundle,
    // This component is used for collision detection, defining the physical boundary of the wall.
    collider: Collider,
}

#[derive(Component)]
// The health value decreases when the brick is hit, and when it reaches zero, the brick can be destroyed.
struct Brick {
    // i8 = small integer - indicates how much damage the brick can withstand before breaking.
    health: i8,
}

// Resources are global data accessible throughout your game, like a global score or game settings. 
// Clone and Copy let you duplicate this data easily.
#[derive(Resource, Clone, Copy)]
struct Scoreboard {
    score: usize,
}

// Useful for global data that needs a default state and direct access to inner data.
#[derive(Resource, Default, Deref, DerefMut)]
struct CollisionSound(Handle<AudioSource>);

fn setup(
    // commands is used to spawn entities (like the camera, paddle, balls, walls) and insert resources (like sounds).
    mut commands: Commands,
    // provides access to the functionality needed to load external assets, like audio and images, into the game 
    asset_server: Res<AssetServer>
    ) {
    //camera
    commands.spawn(Camera2dBundle::default());

    //sound
    let ball_collision_sound = asset_server.load("sounds/breakout_collision.ogg");
    commands.insert_resource(CollisionSound(ball_collision_sound));

    //paddle
    commands.spawn((
        // set up the visual appearance of the paddle
        SpriteBundle {
            transform: Transform {
                translation: vec3(0., PADDLE_START_Y, 0.),
                ..default()
            },
            sprite: Sprite {
                color: PADDLE_COLOR,
                custom_size: Some(PADDLE_SIZE),
                ..default()
            },
            ..default()
        },
        Paddle,
        Collider { size: PADDLE_SIZE },
    ));

    //ball
    let ball_tex = asset_server.load("textures/circle.png");
    
    // Initialize the random number generator using thread_rng function
    let mut rng = thread_rng();

    // for _ in 0..1_000 {
        // Generate random initial direction
        // Generates a random floating-point number between 0.0 and approximately 6.28 (2 pi)
        let random_angle = rng.gen_range(0.0..std::f32::consts::TAU); // TAU is 2*PI = represnet a full rotation in radians
        // set random movement direction for an object in a game
        let random_direction = Vec2::new(random_angle.cos(), random_angle.sin());
        let random_color = Color::rgba(
            rng.gen_range(0.0..1.0),
            rng.gen_range(0.0..1.0),
            rng.gen_range(0.0..1.0),
            1.0, // alpha value, you can randomize this too if you want
        );

        commands.spawn((
            SpriteBundle {
                transform: Transform {
                    translation: BALL_STARTING_POSITION,
                    ..Default::default()
                },
                sprite: Sprite {
                    color: random_color,
                    custom_size: Some(BALL_SIZE),
                    ..Default::default()
                },
                // creates a copy of a value
                // create a new instance of the texture handle ball_tex. 
                // This is necessary because you're using the texture for multiple sprites, and each sprite needs its own handle to the texture.
                texture: ball_tex.clone(),
                ..Default::default()
            },
            Ball { size: BALL_SIZE },
            // have both magnitude and direction
            Velocity(BALL_SPEED * random_direction),
        ));
    

    //walls
    {
        let vertical_wall_size = vec2(WALL_THICKNESS, WALL_BLOCK_HEIGHT + WALL_THICKNESS);
        let horizontal_wall_size = vec2(WALL_BLOCK_WIDTH + WALL_THICKNESS, WALL_THICKNESS);
        //left wall
        commands.spawn(WallBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: vec3(LEFT_WALL, 0.0, 0.0),
                    ..default()
                },
                sprite: Sprite {
                    color: WALL_COLOR,
                    // In Rust, Option is a special type used for values that can either be something (Some) or nothing (None). It's commonly used when a value may or may not be present.
                    // Unlike some other programming languages that use null values, Rust uses Option to handle the absence of a value more safely and clearly
                    // The custom_size field is designed to optionally accept a size.
                    // When you have a size to provide, you wrap it in Some. This tells the program, "Here is the size I want to use."
                    custom_size: Some(vertical_wall_size),
                    ..default()
                },
                ..default()
            },
            // Imagine a soccer ball and a wall. If the wall is just a picture (without a Collider), the ball would go through it as if the wall isn't there. 
            // But if the wall is solid (has a Collider), the ball will bounce off it when they collide. That's what adding a Collider in your game code does - it makes the walls solid for game physics.
            // Adding a Collider to the walls in your game is like making them solid and interactive.
            collider: Collider {
                size: vertical_wall_size,
            },
        });

        //right wall
        commands.spawn(WallBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: vec3(RIGHT_WALL, 0.0, 0.0),
                    ..default()
                },
                sprite: Sprite {
                    color: WALL_COLOR,
                    custom_size: Some(vertical_wall_size),
                    ..default()
                },
                ..default()
            },
            collider: Collider {
                size: vertical_wall_size,
            },
        });

        //bottom wall
        commands.spawn(WallBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: vec3(0.0, BOTTOM_WALL, 0.0),
                    ..default()
                },
                sprite: Sprite {
                    color: WALL_COLOR,
                    custom_size: Some(horizontal_wall_size),
                    ..default()
                },
                ..default()
            },
            collider: Collider {
                size: horizontal_wall_size,
            },
        });

        //top wall
        commands.spawn(WallBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: vec3(0.0, TOP_WALL, 0.0),
                    ..default()
                },
                sprite: Sprite {
                    color: WALL_COLOR,
                    custom_size: Some(horizontal_wall_size),
                    ..default()
                },
                ..default()
            },
            collider: Collider {
                size: horizontal_wall_size,
            },
        });
    }

    //bricks
    {
        let offset_x = LEFT_WALL + GAP_BETWEEN_BRICKS_AND_SIDES + BRICK_SIZE.x * 0.5;
        let offset_y = BOTTOM_WALL + GAP_BETWEEN_PADDLE_AND_BRICKS + BRICK_SIZE.y * 0.5;

        let bricks_total_width = (RIGHT_WALL - LEFT_WALL) - 2. * GAP_BETWEEN_BRICKS_AND_SIDES;
        let bricks_total_height = (TOP_WALL - BOTTOM_WALL)
            - GAP_BETWEEN_BRICKS_AND_CEILING
            - GAP_BETWEEN_PADDLE_AND_BRICKS;

        // floor() rounds down the result to the nearest whole number
        // i32 converts the result to a 32-bit integer
        let rows = (bricks_total_height / (BRICK_SIZE.y + GAP_BETWEEN_BRICKS)).floor() as i32;
        let columns = (bricks_total_width / (BRICK_SIZE.x + GAP_BETWEEN_BRICKS)).floor() as i32;

        for row in 0..rows {
            for column in 0..columns {
                let brick_pos = vec2(
                    // column as f32: This converts the column number (which is an integer) to a floating-point number
                    offset_x + column as f32 * (BRICK_SIZE.x + GAP_BETWEEN_BRICKS),
                    offset_y + row as f32 * (BRICK_SIZE.y + GAP_BETWEEN_BRICKS),
                );

                commands.spawn((
                    SpriteBundle {
                        transform: Transform {
                            // extend(0.0) adds a z-coordinate (depth), which is required for a 3D transform but typically 0.0 in 2D games
                            translation: brick_pos.extend(0.0),
                            ..default()
                        },
                        sprite: Sprite {
                            color: BRICK_COLOR,
                            custom_size: Some(BRICK_SIZE),
                            ..default()
                        },
                        ..default()
                    },
                    Brick { health: 1 },
                    // a Collider is used to define the physical shape of an entity for the purpose of collision detection
                    // Static by Default: Without additional components, a Collider in Bevy doesn't make an entity dynamic. It means that the entity won't move or react to physical forces on its own; it just has a defined shape for collision purposes.
                    Collider { size: BRICK_SIZE },
                ));
            }
        }
    }

    //Scoreboard
    // TextBundle - A bundle of components used in Bevy for creating text-based UI elements.
    // TextBundle::from_sections is a function used to create text entities that consist of multiple parts or "sections." 
    commands.spawn((TextBundle::from_sections([
        // This part creates two pieces of text.
        // First Piece ("Score: "): This is just the word "Score: ".
        TextSection::new(
            "Score: ",
            TextStyle {
                font_size: SCOREBOARD_FONT_SIZE,
                color: TEXT_COLOR,
                ..default()
            },
        ),
        TextSection::from_style(TextStyle {
            font_size: SCOREBOARD_FONT_SIZE,
            color: SCORE_COLOR,
            ..default()
        }),
    ])
    .with_style(Style {
        position_type: PositionType::Absolute,
        top: SCOREBOARD_TEXT_PADDING,
        left: SCOREBOARD_TEXT_PADDING,
        ..default()
    }),));
}

fn move_paddle(
    // allows the function to access player input. It checks which keys are pressed.
    input: Res<Input<KeyCode>>,
    // provides access to the game's timing information, like the duration of the current frame. 
    time_step: Res<FixedTime>,
    // find the entity that represents the paddle and get its Transform component
    mut query: Query<&mut Transform, With<Paddle>>,
) {
    // gets the Transform component of the paddle entity
    let mut paddle_transform = query.single_mut();

    let mut direction = 0.0;
    if input.pressed(KeyCode::A) {
        direction -= 1.0;
    }
    if input.pressed(KeyCode::D) {
        direction += 1.0;
    }

    // calculates the new horizontal position (x coordinate) for the paddle. 
    // time_step.period.as_secs_f32(): This gives the duration of the current frame in seconds as a floating-point number. In other words, it tells you how much time has passed since the last frame.
    let mut new_x =
        // multiply PADDLE_SPEED (how fast) by time_step.period.as_secs_f32() (how much time has passed) to calculate how far the paddle should move in this specific frame.
        // For example, if PADDLE_SPEED is 100 units/second, and your frame time is 1/60th of a second, then in one frame, the paddle should move 100×1/60units.
        paddle_transform.translation.x + direction * PADDLE_SPEED * time_step.period.as_secs_f32();

    // ensure that the paddle doesn't move beyond the boundaries of the game area
    // The min function is used to compare the current new_x value with the calculated right boundary position
    // If new_x is less than the right boundary (meaning the paddle is within the bounds), new_x remains unchanged.
    // If new_x is greater (meaning the paddle would go past the right boundary), new_x is set to the right boundary value to prevent it from going too far.
    new_x = new_x.min(RIGHT_WALL - (WALL_THICKNESS + PADDLE_SIZE.x) * 0.5);
    new_x = new_x.max(LEFT_WALL + (WALL_THICKNESS + PADDLE_SIZE.x) * 0.5);

    paddle_transform.translation.x = new_x;
}

// applies to all entities in your game world that have both a Transform component and a Velocity component.
fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>, time_step: Res<FixedTime>) {
    // dt (delta time) holds the amount of time that has passed since the last frame/update
    let dt = time_step.period.as_secs_f32();
    for (mut transform, velocity) in &mut query {
        // distances = velocity * time
        transform.translation.x += velocity.x * dt;
        transform.translation.y += velocity.y * dt;
    }
}

fn check_ball_collisions(
    mut commands: Commands,
    mut score: ResMut<Scoreboard>,
    collision_sound: Res<CollisionSound>,
    // get entities that have all three components: Velocity, Transform, and Ball.
    mut ball_query: Query<(&mut Velocity, &Transform, &Ball)>,
    // Entity: This retrieves the entity's ID. It's useful for performing operations on the entity itself, like despawning
    // Option<&mut Brick>- This is an optional component. It means this query will include entities even if they don't have a Brick component.
    // If an entity has a Brick component, it provides mutable access to it, allowing you to modify the Brick (like changing its health).
    mut collider_query: Query<(Entity, &Transform, &Collider, Option<&mut Brick>)>, // Note the mutability for Brick
) {
    for (mut ball_velocity, ball_transform, ball) in &mut ball_query {
        for (other_entity, transform, other, opt_brick) in &mut collider_query {
            // The bevy::sprite::collide_aabb::collide function in Rust performs simple AABB collision detection
            // pub fn collide(
            //     a_pos: Vec3,
            //     a_size: Vec2,
            //     b_pos: Vec3,
            //     b_size: Vec2
            // ) -> Option<Collision>
            // 1. Determine the distance between the centers of the two entities.
            // 2. Compare this distance to the combined sizes of the entities. For circular objects, this would be the radii; for rectangular objects, you might use half the width/height.
            // 3. If the distance is less than the combined sizes, a collision is occurring.
            let collision = collide(
                // Position of the First Entity - The current position of the ball
                ball_transform.translation,
                // Size of the First Entity
                ball.size,
                // Position of the Second Entity - The position of the other entity (like a brick or wall)
                transform.translation,
                // Size of the Second Entity - The size of the other entity.
                other.size,
            );

            let mut reflect_x = false;
            let mut reflect_y = false;
            // If a collision is detected, this block determines from which side the collision occurred (left, right, top, bottom, or inside).
            if let Some(collision) = collision {
                match collision {
                    // If the ball hits something on its left side, check if the ball is moving to the right (ball_velocity.x > 0.0). 
                    // If it is, set reflect_x to true. 
                    Collision::Left => reflect_x = ball_velocity.x > 0.0,
                    // If the ball hits something on its right side, check if the ball is moving to the left (ball_velocity.x < 0.0). 
                    // If it is, set reflect_x to true.
                    Collision::Right => reflect_x = ball_velocity.x < 0.0,
                    Collision::Top => reflect_y = ball_velocity.y < 0.0,
                    Collision::Bottom => reflect_y = ball_velocity.y > 0.0,
                    Collision::Inside => { /* do nothing */ }
                }

                if reflect_x {
                // If the ball should bounce (for example, it hit the left side and was moving right), reflect_x is set to true.
                // When reflect_x is true, the code then reverses the ball's horizontal velocity (ball_velocity.x *= -1;). 
                // This reversal makes the ball start moving in the opposite direction, simulating a bounce.
                    ball_velocity.x *= -1.;
                }
                if reflect_y {
                    ball_velocity.y *= -1.;
                }

                if let Some(mut brick) = opt_brick {
                    score.score += 1;
                    // The health of the brick is then decreased by 1. 
                    // However, to avoid negative health values, the max(0) method ensures that the health doesn't drop below zero
                    // This line effectively says, "Reduce the brick's health by one, but if it drops below zero, just set it to zero."
                    brick.health = (brick.health - 1).max(0);


                    // checks if the brick's health is now zero or less. If it is, the brick needs to be removed from the game.
                    if brick.health <= 0 {
                        commands.entity(other_entity).despawn(); // Despawn the Brick if health is 0 or less
                    }
                }

                // play sound
                // commands.spawn(AudioBundle {
                //     source: collision_sound.clone(),
                //     settings: PlaybackSettings::DESPAWN,
                // });
            }
        }
    }
}

fn update_scoreboard(score: Res<Scoreboard>, mut query: Query<&mut Text>) {
    let mut text = query.single_mut();
    // updates the second section of the Text component with the current game score.
    // The scoreboard text is assumed to be split into sections, with the first section likely being static text like "Score: " and the second section (sections[1]) being the part that displays the actual numeric score.
    text.sections[1].value = score.score.to_string();
}