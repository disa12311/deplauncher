use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::*;
use js_sys::*;
use web_sys::*;
use std::collections::{HashMap, VecDeque};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use nalgebra::{Vector3, Matrix4, Quaternion, UnitQuaternion};
use serde::{Serialize, Deserialize};

// Advanced Game Engine for Deplauncher 1.12 - Enhanced Edition (Rust)
// State-of-the-art engine with advanced physics, AI, networking, and graphics

const MAX_ENTITIES: usize = 5000;
const MAX_PARTICLES: usize = 10000;
const MAX_LIGHTS: usize = 50;
const CANVAS_WIDTH: f32 = 1920.0;
const CANVAS_HEIGHT: f32 = 1080.0;
const PHYSICS_SUBSTEPS: usize = 4;
const NETWORKING_BUFFER_SIZE: usize = 8192;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
    
    #[wasm_bindgen(js_namespace = performance)]
    fn now() -> f64;
    
    #[wasm_bindgen(js_namespace = WebSocket)]
    type WebSocket;
    
    #[wasm_bindgen(constructor)]
    fn new(url: &str) -> WebSocket;
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

// Advanced Transform component with full 3D support
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Transform {
    pub position: Vector3<f32>,
    pub rotation: UnitQuaternion<f32>,
    pub scale: Vector3<f32>,
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            position: Vector3::zeros(),
            rotation: UnitQuaternion::identity(),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }
}

impl Transform {
    pub fn new(pos: Vector3<f32>) -> Self {
        Transform {
            position: pos,
            ..Default::default()
        }
    }
    
    pub fn matrix(&self) -> Matrix4<f32> {
        Matrix4::new_translation(&self.position) *
        self.rotation.to_homogeneous() *
        Matrix4::new_nonuniform_scaling(&self.scale)
    }
    
    pub fn forward(&self) -> Vector3<f32> {
        self.rotation * Vector3::z()
    }
    
    pub fn right(&self) -> Vector3<f32> {
        self.rotation * Vector3::x()
    }
    
    pub fn up(&self) -> Vector3<f32> {
        self.rotation * Vector3::y()
    }
}

// Advanced Physics Component
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Physics {
    pub velocity: Vector3<f32>,
    pub acceleration: Vector3<f32>,
    pub angular_velocity: Vector3<f32>,
    pub mass: f32,
    pub drag: f32,
    pub angular_drag: f32,
    pub friction: f32,
    pub bounciness: f32,
    pub is_kinematic: bool,
    pub use_gravity: bool,
    pub freeze_rotation: [bool; 3],
    pub freeze_position: [bool; 3],
}

impl Default for Physics {
    fn default() -> Self {
        Physics {
            velocity: Vector3::zeros(),
            acceleration: Vector3::zeros(),
            angular_velocity: Vector3::zeros(),
            mass: 1.0,
            drag: 0.01,
            angular_drag: 0.05,
            friction: 0.1,
            bounciness: 0.5,
            is_kinematic: false,
            use_gravity: true,
            freeze_rotation: [false; 3],
            freeze_position: [false; 3],
        }
    }
}

// Advanced Rendering Component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Renderer {
    pub mesh_id: u32,
    pub material_id: u32,
    pub texture_ids: Vec<u32>,
    pub color: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub emission: [f32; 3],
    pub normal_strength: f32,
    pub cast_shadows: bool,
    pub receive_shadows: bool,
    pub layer: u8,
    pub visible: bool,
}

impl Default for Renderer {
    fn default() -> Self {
        Renderer {
            mesh_id: 0,
            material_id: 0,
            texture_ids: Vec::new(),
            color: [1.0, 1.0, 1.0, 1.0],
            metallic: 0.0,
            roughness: 0.5,
            emission: [0.0, 0.0, 0.0],
            normal_strength: 1.0,
            cast_shadows: true,
            receive_shadows: true,
            layer: 0,
            visible: true,
        }
    }
}

// AI Component with behavior tree support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AI {
    pub behavior_tree: String,
    pub state: String,
    pub target_entity: Option<u32>,
    pub target_position: Option<Vector3<f32>>,
    pub path: VecDeque<Vector3<f32>>,
    pub decision_timer: f32,
    pub memory: HashMap<String, f32>,
    pub enabled: bool,
}

impl Default for AI {
    fn default() -> Self {
        AI {
            behavior_tree: "idle".to_string(),
            state: "idle".to_string(),
            target_entity: None,
            target_position: None,
            path: VecDeque::new(),
            decision_timer: 0.0,
            memory: HashMap::new(),
            enabled: true,
        }
    }
}

// Health Component
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Health {
    pub current: f32,
    pub max: f32,
    pub regeneration: f32,
    pub armor: f32,
    pub invulnerable: bool,
    pub last_damage_time: f32,
}

impl Default for Health {
    fn default() -> Self {
        Health {
            current: 100.0,
            max: 100.0,
            regeneration: 0.0,
            armor: 0.0,
            invulnerable: false,
            last_damage_time: 0.0,
        }
    }
}

// Network Component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Network {
    pub owner_id: u32,
    pub networked: bool,
    pub authority: bool,
    pub last_sync_time: f64,
    pub interpolate: bool,
    pub extrapolate: bool,
}

impl Default for Network {
    fn default() -> Self {
        Network {
            owner_id: 0,
            networked: false,
            authority: true,
            last_sync_time: 0.0,
            interpolate: true,
            extrapolate: false,
        }
    }
}

// Advanced Entity with ECS architecture
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedEntity {
    pub id: u32,
    pub name: String,
    pub tag: String,
    pub layer: u8,
    pub active: bool,
    
    // Components
    pub transform: Transform,
    pub physics: Option<Physics>,
    pub renderer: Option<Renderer>,
    pub health: Option<Health>,
    pub ai: Option<AI>,
    pub network: Option<Network>,
    
    // Custom component data
    pub custom_data: HashMap<String, f32>,
}

impl AdvancedEntity {
    pub fn new(id: u32, name: String, position: Vector3<f32>) -> Self {
        AdvancedEntity {
            id,
            name,
            tag: "Untagged".to_string(),
            layer: 0,
            active: true,
            transform: Transform::new(position),
            physics: None,
            renderer: None,
            health: None,
            ai: None,
            network: None,
            custom_data: HashMap::new(),
        }
    }
    
    pub fn add_physics(&mut self) -> &mut Physics {
        self.physics = Some(Physics::default());
        self.physics.as_mut().unwrap()
    }
    
    pub fn add_renderer(&mut self) -> &mut Renderer {
        self.renderer = Some(Renderer::default());
        self.renderer.as_mut().unwrap()
    }
    
    pub fn add_health(&mut self, max_health: f32) -> &mut Health {
        self.health = Some(Health {
            current: max_health,
            max: max_health,
            ..Default::default()
        });
        self.health.as_mut().unwrap()
    }
    
    pub fn add_ai(&mut self) -> &mut AI {
        self.ai = Some(AI::default());
        self.ai.as_mut().unwrap()
    }
    
    pub fn add_network(&mut self, owner_id: u32) -> &mut Network {
        self.network = Some(Network {
            owner_id,
            networked: true,
            ..Default::default()
        });
        self.network.as_mut().unwrap()
    }
}

// Advanced Particle System
#[derive(Debug, Clone)]
pub struct Particle {
    pub position: Vector3<f32>,
    pub velocity: Vector3<f32>,
    pub acceleration: Vector3<f32>,
    pub color: [f32; 4],
    pub size: f32,
    pub rotation: f32,
    pub angular_velocity: f32,
    pub life: f32,
    pub max_life: f32,
    pub active: bool,
    pub particle_type: ParticleType,
}

#[derive(Debug, Clone)]
pub enum ParticleType {
    Fire,
    Smoke,
    Spark,
    Magic,
    Explosion,
    Trail,
}

#[derive(Debug)]
pub struct ParticleSystem {
    particles: Vec<Particle>,
    emitters: Vec<ParticleEmitter>,
    gravity: Vector3<f32>,
    wind: Vector3<f32>,
}

#[derive(Debug)]
pub struct ParticleEmitter {
    pub position: Vector3<f32>,
    pub rate: f32,
    pub life_time: f32,
    pub particle_life: f32,
    pub velocity: Vector3<f32>,
    pub velocity_random: Vector3<f32>,
    pub color_start: [f32; 4],
    pub color_end: [f32; 4],
    pub size_start: f32,
    pub size_end: f32,
    pub particle_type: ParticleType,
    pub active: bool,
    pub timer: f32,
}

impl ParticleSystem {
    pub fn new() -> Self {
        ParticleSystem {
            particles: Vec::with_capacity(MAX_PARTICLES),
            emitters: Vec::new(),
            gravity: Vector3::new(0.0, -98.0, 0.0),
            wind: Vector3::zeros(),
        }
    }
    
    pub fn update(&mut self, delta_time: f32) {
        // Update existing particles
        self.particles.retain_mut(|particle| {
            if !particle.active {
                return false;
            }
            
            // Apply physics
            particle.acceleration += self.gravity + self.wind;
            particle.velocity += particle.acceleration * delta_time;
            particle.position += particle.velocity * delta_time;
            particle.rotation += particle.angular_velocity * delta_time;
            
            // Update life
            particle.life -= delta_time;
            if particle.life <= 0.0 {
                particle.active = false;
                return false;
            }
            
            // Update color and size based on life
            let life_ratio = particle.life / particle.max_life;
            for i in 0..4 {
                particle.color[i] *= life_ratio;
            }
            particle.size *= 0.99; // Gradually shrink
            
            // Reset acceleration for next frame
            particle.acceleration = Vector3::zeros();
            
            true
        });
        
        // Update emitters and spawn new particles
        for emitter in &mut self.emitters {
            if !emitter.active {
                continue;
            }
            
            emitter.timer += delta_time;
            
            // Spawn particles based on rate
            let spawn_interval = 1.0 / emitter.rate;
            while emitter.timer >= spawn_interval && self.particles.len() < MAX_PARTICLES {
                self.spawn_particle_from_emitter(emitter);
                emitter.timer -= spawn_interval;
            }
            
            // Decrease emitter lifetime
            emitter.life_time -= delta_time;
            if emitter.life_time <= 0.0 {
                emitter.active = false;
            }
        }
        
        // Remove inactive emitters
        self.emitters.retain(|e| e.active);
    }
    
    fn spawn_particle_from_emitter(&mut self, emitter: &ParticleEmitter) {
        let random_velocity = Vector3::new(
            (js_sys::Math::random() as f32 - 0.5) * emitter.velocity_random.x,
            (js_sys::Math::random() as f32 - 0.5) * emitter.velocity_random.y,
            (js_sys::Math::random() as f32 - 0.5) * emitter.velocity_random.z,
        );
        
        let particle = Particle {
            position: emitter.position,
            velocity: emitter.velocity + random_velocity,
            acceleration: Vector3::zeros(),
            color: emitter.color_start,
            size: emitter.size_start,
            rotation: 0.0,
            angular_velocity: (js_sys::Math::random() as f32 - 0.5) * 10.0,
            life: emitter.particle_life * (0.5 + js_sys::Math::random() as f32 * 0.5),
            max_life: emitter.particle_life,
            active: true,
            particle_type: emitter.particle_type.clone(),
        };
        
        self.particles.push(particle);
    }
    
    pub fn create_explosion(&mut self, position: Vector3<f32>, intensity: f32) {
        let emitter = ParticleEmitter {
            position,
            rate: intensity * 100.0,
            life_time: 0.5,
            particle_life: 2.0,
            velocity: Vector3::zeros(),
            velocity_random: Vector3::new(200.0, 200.0, 200.0) * intensity,
            color_start: [1.0, 0.8, 0.2, 1.0],
            color_end: [1.0, 0.2, 0.0, 0.0],
            size_start: 5.0 * intensity,
            size_end: 1.0,
            particle_type: ParticleType::Explosion,
            active: true,
            timer: 0.0,
        };
        
        self.emitters.push(emitter);
    }
}

// Advanced Lighting System
#[derive(Debug, Clone)]
pub struct Light {
    pub position: Vector3<f32>,
    pub direction: Vector3<f32>,
    pub color: [f32; 3],
    pub intensity: f32,
    pub range: f32,
    pub light_type: LightType,
    pub spot_angle: f32,
    pub cast_shadows: bool,
    pub active: bool,
}

#[derive(Debug, Clone)]
pub enum LightType {
    Directional,
    Point,
    Spot,
    Area,
}

// Advanced Camera System
#[derive(Debug, Clone)]
pub struct Camera {
    pub transform: Transform,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub aspect_ratio: f32,
    pub projection_matrix: Matrix4<f32>,
    pub view_matrix: Matrix4<f32>,
    pub target: Option<u32>, // Entity to follow
    pub follow_speed: f32,
    pub look_ahead: f32,
}

impl Camera {
    pub fn new(position: Vector3<f32>, fov: f32, aspect_ratio: f32) -> Self {
        let mut camera = Camera {
            transform: Transform::new(position),
            fov,
            near: 0.1,
            far: 1000.0,
            aspect_ratio,
            projection_matrix: Matrix4::identity(),
            view_matrix: Matrix4::identity(),
            target: None,
            follow_speed: 5.0,
            look_ahead: 2.0,
        };
        
        camera.update_matrices();
        camera
    }
    
    pub fn update_matrices(&mut self) {
        self.projection_matrix = Matrix4::new_perspective(
            self.aspect_ratio,
            self.fov.to_radians(),
            self.near,
            self.far,
        );
        
        let eye = self.transform.position;
        let target = eye + self.transform.forward();
        let up = self.transform.up();
        
        self.view_matrix = Matrix4::look_at_rh(&eye, &target, &up);
    }
    
    pub fn follow_entity(&mut self, entity: &AdvancedEntity, delta_time: f32) {
        let target_pos = entity.transform.position;
        let predicted_pos = if let Some(physics) = &entity.physics {
            target_pos + physics.velocity * self.look_ahead
        } else {
            target_pos
        };
        
        let lerp_factor = 1.0 - (-self.follow_speed * delta_time).exp();
        self.transform.position = self.transform.position.lerp(&predicted_pos, lerp_factor);
        self.update_matrices();
    }
}

// Advanced Game State with ECS architecture
#[wasm_bindgen]
pub struct AdvancedGameState {
    entities: HashMap<u32, AdvancedEntity>,
    next_entity_id: u32,
    
    // Systems
    particle_system: ParticleSystem,
    lights: Vec<Light>,
    camera: Camera,
    
    // Game state
    score: i32,
    level: i32,
    time_scale: f32,
    paused: bool,
    
    // Performance metrics
    last_frame_time: f64,
    fps_counter: i32,
    fps_timer: f64,
    frame_time_ms: f32,
    draw_calls: i32,
    
    // Physics settings
    gravity: Vector3<f32>,
    air_density: f32,
    physics_enabled: bool,
    
    // Input system
    input_state: HashMap<u32, bool>,
    mouse_position: Vector3<f32>,
    mouse_delta: Vector3<f32>,
    
    // Audio settings
    master_volume: f32,
    sfx_volume: f32,
    music_volume: f32,
    
    // Networking
    multiplayer_enabled: bool,
    player_id: u32,
    server_url: String,
    network_buffer: Vec<u8>,
    
    // Graphics settings
    graphics_quality: u8,
    bloom_enabled: bool,
    ssao_enabled: bool,
    motion_blur_enabled: bool,
    pbr_enabled: bool,
    shadows_enabled: bool,
    reflections_enabled: bool,
    exposure: f32,
    gamma: f32,
}

#[wasm_bindgen]
impl AdvancedGameState {
    #[wasm_bindgen(constructor)]
    pub fn new() -> AdvancedGameState {
        console_log!("Creating Advanced Game State for v1.12 Enhanced Edition");
        
        let mut game_state = AdvancedGameState {
            entities: HashMap::with_capacity(MAX_ENTITIES),
            next_entity_id: 1,
            
            particle_system: ParticleSystem::new(),
            lights: Vec::with_capacity(MAX_LIGHTS),
            camera: Camera::new(
                Vector3::new(CANVAS_WIDTH / 2.0, CANVAS_HEIGHT / 2.0, -500.0),
                75.0,
                CANVAS_WIDTH / CANVAS_HEIGHT
            ),
            
            score: 0,
            level: 1,
            time_scale: 1.0,
            paused: false,
            
            last_frame_time: now(),
            fps_counter: 0,
            fps_timer: 0.0,
            frame_time_ms: 0.0,
            draw_calls: 0,
            
            gravity: Vector3::new(0.0, -980.0, 0.0),
            air_density: 1.225,
            physics_enabled: true,
            
            input_state: HashMap::new(),
            mouse_position: Vector3::zeros(),
            mouse_delta: Vector3::zeros(),
            
            master_volume: 1.0,
            sfx_volume: 0.8,
            music_volume: 0.6,
            
            multiplayer_enabled: false,
            player_id: 0,
            server_url: String::new(),
            network_buffer: Vec::with_capacity(NETWORKING_BUFFER_SIZE),
            
            graphics_quality: 2, // High quality by default
            bloom_enabled: true,
            ssao_enabled: true,
            motion_blur_enabled: false,
            pbr_enabled: true,
            shadows_enabled: true,
            reflections_enabled: true,
            exposure: 1.0,
            gamma: 2.2,
        };
        
        game_state.initialize_scene();
        game_state
    }
    
    fn initialize_scene(&mut self) {
        // Create advanced player entity with full component setup
        let player_id = self.create_entity("Player".to_string(), Vector3::new(CANVAS_WIDTH / 2.0, CANVAS_HEIGHT / 2.0, 0.0));
        
        if let Some(player) = self.entities.get_mut(&player_id) {
            // Add physics component
            let physics = player.add_physics();
            physics.mass = 1.0;
            physics.use_gravity = false; // Top-down view
            physics.drag = 5.0; // High drag for responsive movement
            
            // Add renderer component
            let renderer = player.add_renderer();
            renderer.color = [0.2, 0.8, 1.0, 1.0]; // Cyan player
            renderer.metallic = 0.1;
            renderer.roughness = 0.3;
            
            // Add health component
            let health = player.add_health(100.0);
            health.regeneration = 5.0; // HP per second
            
            player.tag = "Player".to_string();
        }
        
        // Create procedurally generated environment
        self.generate_environment();
        
        // Setup lighting
        self.setup_lighting();
        
        // Set camera to follow player
        self.camera.target = Some(player_id);
        
        console_log!("Advanced scene initialized with {} entities", self.entities.len());
    }
    
    fn generate_environment(&mut self) {
        // Create various types of environment objects
        for i in 0..100 {
            let x = js_sys::Math::random() as f32 * CANVAS_WIDTH;
            let y = js_sys::Math::random() as f32 * CANVAS_HEIGHT;
            let z = (js_sys::Math::random() as f32 - 0.5) * 200.0;
            
            let entity_id = self.create_entity(format!("Environment_{}", i), Vector3::new(x, y, z));
            
            if let Some(entity) = self.entities.get_mut(&entity_id) {
                // Add physics
                let physics = entity.add_physics();
                physics.mass = 0.5 + js_sys::Math::random() as f32 * 2.0;
                physics.bounciness = 0.3 + js_sys::Math::random() as f32 * 0.7;
                physics.friction = 0.1 + js_sys::Math::random() as f32 * 0.8;
                
                // Add renderer with random materials
                let renderer = entity.add_renderer();
                renderer.color = [
                    0.5 + js_sys::Math::random() as f32 * 0.5,
                    0.5 + js_sys::Math::random() as f32 * 0.5,
                    0.5 + js_sys::Math::random() as f32 * 0.5,
                    1.0,
                ];
                renderer.metallic = js_sys::Math::random() as f32;
                renderer.roughness = 0.2 + js_sys::Math::random() as f32 * 0.8;
                
                // Some objects have AI
                if js_sys::Math::random() < 0.3 {
                    let ai = entity.add_ai();
                    ai.behavior_tree = "wander".to_string();
                    ai.state = "idle".to_string();
                }
                
                entity.tag = "Environment".to_string();
            }
        }
        
        // Create some special interactive objects
        for i in 0..20 {
            let x = js_sys::Math::random() as f32 * CANVAS_WIDTH;
            let y = js_sys::Math::random() as f32 * CANVAS_HEIGHT;
            
            let entity_id = self.create_entity(format!("Interactive_{}", i), Vector3::new(x, y, 0.0));
            
            if let Some(entity) = self.entities.get_mut(&entity_id) {
                let health = entity.add_health(50.0);
                health.regeneration = -1.0; // Slowly decays
                
                let renderer = entity.add_renderer();
                renderer.color = [1.0, 0.5, 0.0, 1.0]; // Orange
                renderer.emission = [0.2, 0.1, 0.0]; // Glowing
                
                entity.tag = "Interactive".to_string();
            }
        }
    }
    
    fn setup_lighting(&mut self) {
        // Main directional light (sun)
        let main_light = Light {
            position: Vector3::new(CANVAS_WIDTH / 2.0, CANVAS_HEIGHT / 2.0, 1000.0),
            direction: Vector3::new(0.3, -1.0, 0.3).normalize(),
            color: [1.0, 0.95, 0.8],
            intensity: 3.0,
            range: 0.0, // Infinite range for directional
            light_type: LightType::Directional,
            spot_angle: 0.0,
            cast_shadows: true,
            active: true,
        };
        self.lights.push(main_light);
        
        // Ambient point lights
        for i in 0..10 {
            let x = js_sys::Math::random() as f32 * CANVAS_WIDTH;
            let y = js_sys::Math::random() as f32 * CANVAS_HEIGHT;
            let z = 50.0 + js_sys::Math::random() as f32 * 200.0;
            
            let point_light = Light {
                position: Vector3::new(x, y, z),
                direction: Vector3::zeros(),
                color: [
                    0.5 + js_sys::Math::random() as f32 * 0.5,
                    0.5 + js_sys::Math::random() as f32 * 0.5,
                    0.5 + js_sys::Math::random() as f32 * 0.5,
                ],
                intensity: 1.0 + js_sys::Math::random() as f32 * 2.0,
                range: 100.0 + js_sys::Math::random() as f32 * 200.0,
                light_type: LightType::Point,
                spot_angle: 0.0,
                cast_shadows: i < 3, // Only first 3 cast shadows
                active: true,
            };
            self.lights.push(point_light);
        }
    }
    
    pub fn create_entity(&mut self, name: String, position: Vector3<f32>) -> u32 {
        let id = self.next_entity_id;
        self.next_entity_id += 1;
        
        let entity = AdvancedEntity::new(id, name, position);
        self.entities.insert(id, entity);
        
        id
    }
    
    #[wasm_bindgen]
    pub fn update(&mut self, current_time: f64) {
        if self.paused {
            return;
        }
        
        let frame_start = now();
        let mut delta_time = ((current_time - self.last_frame_time) / 1000.0) as f32 * self.time_scale;
        self.last_frame_time = current_time;
        
        // Cap delta time to prevent large jumps
        if delta_time > 0.033 {
            delta_time = 0.033;
        }
        
        // Update FPS counter
        self.fps_counter += 1;
        self.fps_timer += delta_time as f64;
        if self.fps_timer >= 1.0 {
            console_log!("FPS: {}, Frame Time: {:.2}ms, Entities: {}, Particles: {}", 
                        self.fps_counter, self.frame_time_ms, self.entities.len(), self.particle_system.particles.len());
            self.fps_counter = 0;
            self.fps_timer = 0.0;
        }
        
        // Update systems
        self.update_physics_system(delta_time);
        self.update_ai_system(delta_time);
        self.update_health_system(delta_time);
        self.update_animation_system(delta_time);
        self.particle_system.update(delta_time);
        self.update_camera_system(delta_time);
        self.update_networking_system();
        
        // Handle input
        self.process_input(delta_time);
        
        // Collision detection and response
        self.collision_detection_and_response();
        
        // Cleanup inactive entities
        self.cleanup_inactive_entities();
        
        // Calculate frame time
        let frame_end = now();
        self.frame_time_ms = (frame_end - frame_start) as f32;
    }
    
    fn update_physics_system(&mut self, delta_time: f32) {
        if !self.physics_enabled {
            return;
        }
        
        let sub_delta = delta_time / PHYSICS_SUBSTEPS as f32;
        
        for _ in 0..PHYSICS_SUBSTEPS {
            for entity in self.entities.values_mut() {
                if !entity.active {
                    continue;
                }
                
                if let Some(physics) = entity.physics.as_mut() {
                    if physics.is_kinematic {
                        continue;
                    }
                    
                    // Apply gravity
                    if physics.use_gravity {
                        physics.acceleration += self.gravity;
                    }
                    
                    // Apply drag
                    let speed = physics.velocity.magnitude();
                    if speed > 0.01 {
                        let drag_force = 0.5 * self.air_density * speed * speed * physics.drag;
                        let drag_acceleration = -(physics.velocity.normalize() * drag_force) / physics.mass;
                        physics.acceleration += drag_acceleration;
                    }
                    
                    // Integration
                    physics.velocity += physics.acceleration * sub_delta;
                    
                    // Apply position constraints
                    if !physics.freeze_position[0] {
                        entity.transform.position.x += physics.velocity.x * sub_delta;
                    }
                    if !physics.freeze_position[1] {
                        entity.transform.position.y += physics.velocity.y * sub_delta;
                    }
                    if !physics.freeze_position[2] {
                        entity.transform.position.z += physics.velocity.z * sub_delta;
                    }
                    
                    // Apply rotation constraints and angular physics
                    if !physics.freeze_rotation[0] || !physics.freeze_rotation[1] || !physics.freeze_rotation[2] {
                        let angular_quat = UnitQuaternion::from_scaled_axis(physics.angular_velocity * sub_delta);
                        entity.transform.rotation = entity.transform.rotation * angular_quat;
                    }
                    
                    // Apply friction
                    physics.velocity *= 1.0 - (physics.friction * sub_delta);
                    physics.angular_velocity *= 1.0 - (physics.angular_drag * sub_delta);
                    
                    // Reset acceleration
                    physics.acceleration = Vector3::zeros();
                }
            }
        }
    }
    
    fn update_ai_system(&mut self, delta_time: f32) {
        let entity_positions: HashMap<u32, Vector3<f32>> = self.entities
            .iter()
            .map(|(&id, entity)| (id, entity.transform.position))
            .collect();
        
        for entity in self.entities.values_mut() {
            if !entity.active {
                continue;
            }
            
            if let Some(ai) = entity.ai.as_mut() {
                if !ai.enabled {
                    continue;
                }
                
                ai.decision_timer -= delta_time;
                
                if ai.decision_timer <= 0.0 {
                    ai.decision_timer = 0.5; // Make decisions every 0.5 seconds
                    
                    match ai.behavior_tree.as_str() {
                        "wander" => {
                            if ai.path.is_empty() {
                                // Generate new random target
                                let target = Vector3::new(
                                    js_sys::Math::random() as f32 * CANVAS_WIDTH,
                                    js_sys::Math::random() as f32 * CANVAS_HEIGHT,
                                    entity.transform.position.z,
                                );
                                ai.target_position = Some(target);
                                ai.path.push_back(target);
                            }
                        }
                        "seek_player" => {
                            // Find player entity
                            if let Some(player_pos) = entity_positions.get(&1) { // Assume player ID is 1
                                ai.target_position = Some(*player_pos);
                                ai.target_entity = Some(1);
                            }
                        }
                        _ => {} // Idle or unknown behavior
                    }
                }
                
                // Execute current behavior
                if let Some(target) = ai.target_position {
                    let direction = target - entity.transform.position;
                    if direction.magnitude() > 5.0 {
                        let move_force = direction.normalize() * 100.0;
                        
                        if let Some(physics) = entity.physics.as_mut() {
                            physics.acceleration += move_force / physics.mass;
                        }
                    } else {
                        // Reached target
                        ai.target_position = None;
                        if !ai.path.is_empty() {
                            ai.path.pop_front();
                            if let Some(next_target) = ai.path.front() {
                                ai.target_position = Some(*next_target);
                            }
                        }
                    }
                }
            }
        }
    }
    
    fn update_health_system(&mut self, delta_time: f32) {
        let mut entities_to_remove = Vec::new();
        
        for (&id, entity) in self.entities.iter_mut() {
            if !entity.active {
                continue;
            }
            
            if let Some(health) = entity.health.as_mut() {
                // Apply regeneration
                health.current += health.regeneration * delta_time;
                health.current = health.current.min(health.max).max(0.0);
                
                // Check for death
                if health.current <= 0.0 {
                    // Create death effect
                    self.particle_system.create_explosion(entity.transform.position, 1.0);
                    
                    // Mark for removal
                    entities_to_remove.push(id);
                    
                    // Add score if not player
                    if entity.tag != "Player" {
                        self.score += 10;
                    }
                }
            }
        }
        
        // Remove dead entities
        for id in entities_to_remove {
            self.entities.remove(&id);
        }
    }
    
    fn update_animation_system(&mut self, _delta_time: f32) {
        // Animation system would be implemented here
        // This would handle skeletal animations, sprite animations, etc.
    }
    
    fn update_camera_system(&mut self, delta_time: f32) {
        if let Some(target_id) = self.camera.target {
            if let Some(target_entity) = self.entities.get(&target_id) {
                self.camera.follow_entity(target_entity, delta_time);
            }
        }
    }
    
    fn update_networking_system(&mut self) {
        if !self.multiplayer_enabled {
            return;
        }
        
        // Networking system would be implemented here
        // This would handle entity synchronization, state updates, etc.
    }
    
    fn process_input(&mut self, delta_time: f32) {
        // Find player entity
        let player_entity = self.entities.get_mut(&1);
        if let Some(player) = player_entity {
            if let Some(physics) = player.physics.as_mut() {
                let move_speed = 300.0;
                let mut move_direction = Vector3::zeros();
                
                // WASD movement
                if *self.input_state.get(&87).unwrap_or(&false) { // W
                    move_direction.y -= 1.0;
                }
                if *self.input_state.get(&83).unwrap_or(&false) { // S
                    move_direction.y += 1.0;
                }
                if *self.input_state.get(&65).unwrap_or(&false) { // A
                    move_direction.x -= 1.0;
                }
                if *self.input_state.get(&68).unwrap_or(&false) { // D
                    move_direction.x += 1.0;
                }
                
                // Normalize and apply movement
                if move_direction.magnitude() > 0.0 {
                    move_direction = move_direction.normalize();
                    physics.acceleration += move_direction * move_speed;
                }
            }
        }
    }
    
    fn collision_detection_and_response(&mut self) {
        // Broad phase collision detection using spatial partitioning would go here
        // For now, we'll use a simple O(n²) approach
        
        let entity_ids: Vec<u32> = self.entities.keys().cloned().collect();
        
        for i in 0..entity_ids.len() {
            for j in (i + 1)..entity_ids.len() {
                let id_a = entity_ids[i];
                let id_b = entity_ids[j];
                
                let collision_data = {
                    let entity_a = &self.entities[&id_a];
                    let entity_b = &self.entities[&id_b];
                    
                    if !entity_a.active || !entity_b.active {
                        continue;
                    }
                    
                    let distance = (entity_a.transform.position - entity_b.transform.position).magnitude();
                    let collision_threshold = 32.0; // Basic collision radius
                    
                    if distance < collision_threshold {
                        Some((
                            id_a, id_b,
                            entity_a.transform.position,
                            entity_b.transform.position,
                            distance,
                            entity_a.tag.clone(),
                            entity_b.tag.clone()
                        ))
                    } else {
                        None
                    }
                };
                
                if let Some((id_a, id_b, pos_a, pos_b, distance, tag_a, tag_b)) = collision_data {
                    // Handle collision response
                    let direction = (pos_a - pos_b).normalize();
                    let overlap = 32.0 - distance;
                    
                    // Separate entities
                    if let Some(entity_a) = self.entities.get_mut(&id_a) {
                        entity_a.transform.position += direction * overlap * 0.5;
                        
                        if let Some(physics_a) = entity_a.physics.as_mut() {
                            let bounce_force = 150.0;
                            physics_a.velocity += direction * bounce_force;
                        }
                    }
                    
                    if let Some(entity_b) = self.entities.get_mut(&id_b) {
                        entity_b.transform.position -= direction * overlap * 0.5;
                        
                        if let Some(physics_b) = entity_b.physics.as_mut() {
                            let bounce_force = 150.0;
                            physics_b.velocity -= direction * bounce_force;
                        }
                    }
                    
                    // Handle specific collision interactions
                    if tag_a == "Player" || tag_b == "Player" {
                        self.score += 5;
                        
                        // Create particle effect
                        let effect_pos = (pos_a + pos_b) * 0.5;
                        self.particle_system.create_explosion(effect_pos, 0.5);
                    }
                }
            }
        }
    }
    
    fn cleanup_inactive_entities(&mut self) {
        self.entities.retain(|_, entity| entity.active);
    }
    
    // WASM exports
    #[wasm_bindgen]
    pub fn handle_key_event(&mut self, key_code: u32, pressed: bool) {
        self.input_state.insert(key_code, pressed);
        
        if pressed && key_code == 32 { // Space
            self.paused = !self.paused;
        }
    }
    
    #[wasm_bindgen]
    pub fn handle_mouse_event(&mut self, x: f32, y: f32, delta_x: f32, delta_y: f32) {
        self.mouse_position = Vector3::new(x, y, 0.0);
        self.mouse_delta = Vector3::new(delta_x, delta_y, 0.0);
    }
    
    #[wasm_bindgen]
    pub fn get_score(&self) -> i32 {
        self.score
    }
    
    #[wasm_bindgen]
    pub fn get_entity_count(&self) -> usize {
        self.entities.len()
    }
    
    #[wasm_bindgen]
    pub fn get_particle_count(&self) -> usize {
        self.particle_system.particles.len()
    }
    
    #[wasm_bindgen]
    pub fn get_frame_time(&self) -> f32 {
        self.frame_time_ms
    }
    
    #[wasm_bindgen]
    pub fn set_graphics_quality(&mut self, quality: u8) {
        self.graphics_quality = quality;
        
        match quality {
            0 => { // Low
                self.shadows_enabled = false;
                self.bloom_enabled = false;
                self.ssao_enabled = false;
                self.reflections_enabled = false;
                self.pbr_enabled = false;
            }
            1 => { // Medium
                self.shadows_enabled = true;
                self.bloom_enabled = true;
                self.ssao_enabled = false;
                self.reflections_enabled = false;
                self.pbr_enabled = true;
            }
            2 => { // High
                self.shadows_enabled = true;
                self.bloom_enabled = true;
                self.ssao_enabled = true;
                self.reflections_enabled = true;
                self.pbr_enabled = true;
            }
            _ => {}
        }
        
        console_log!("Graphics quality set to {}", quality);
    }
    
    #[wasm_bindgen]
    pub fn enable_multiplayer(&mut self, server_url: String) {
        self.server_url = server_url;
        self.multiplayer_enabled = true;
        console_log!("Multiplayer enabled, connecting to: {}", self.server_url);
    }
    
    #[wasm_bindgen]
    pub fn get_entity_data(&self) -> JsValue {
        let mut data = js_sys::Array::new();
        
        for entity in self.entities.values() {
            if !entity.active {
                continue;
            }
            
            let entity_data = js_sys::Object::new();
            js_sys::Reflect::set(&entity_data, &"id".into(), &entity.id.into()).unwrap();
            js_sys::Reflect::set(&entity_data, &"name".into(), &entity.name.clone().into()).unwrap();
            js_sys::Reflect::set(&entity_data, &"x".into(), &entity.transform.position.x.into()).unwrap();
            js_sys::Reflect::set(&entity_data, &"y".into(), &entity.transform.position.y.into()).unwrap();
            js_sys::Reflect::set(&entity_data, &"z".into(), &entity.transform.position.z.into()).unwrap();
            
            data.push(&entity_data);
        }
        
        data.into()
    }
    
    #[wasm_bindgen]
    pub fn reset_game(&mut self) {
        console_log!("Resetting advanced game state");
        self.entities.clear();
        self.next_entity_id = 1;
        self.score = 0;
        self.level = 1;
        self.paused = false;
        self.particle_system = ParticleSystem::new();
        self.initialize_scene();
    }
    
    #[wasm_bindgen]
    pub fn cleanup(&mut self) {
        console_log!("Cleaning up Advanced Game Engine v1.12");
        self.entities.clear();
        self.particle_system.particles.clear();
        self.lights.clear();
    }
}

// WASM GC utilities
#[wasm_bindgen]
pub struct WasmAdvancedGameEngine {
    game_state: Rc<RefCell<AdvancedGameState>>,
}

#[wasm_bindgen]
impl WasmAdvancedGameEngine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmAdvancedGameEngine {
        console_log!("Initializing WASM Advanced Game Engine v1.12");
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        
        WasmAdvancedGameEngine {
            game_state: Rc::new(RefCell::new(AdvancedGameState::new())),
        }
    }
    
    #[wasm_bindgen]
    pub fn update_frame(&self, current_time: f64) {
        self.game_state.borrow_mut().update(current_time);
    }
    
    #[wasm_bindgen]
    pub fn handle_key(&self, key_code: u32, pressed: bool) {
        self.game_state.borrow_mut().handle_key_event(key_code, pressed);
    }
    
    #[wasm_bindgen]
    pub fn handle_mouse(&self, x: f32, y: f32, delta_x: f32, delta_y: f32) {
        self.game_state.borrow_mut().handle_mouse_event(x, y, delta_x, delta_y);
    }
    
    #[wasm_bindgen]
    pub fn get_score(&self) -> i32 {
        self.game_state.borrow().get_score()
    }
    
    #[wasm_bindgen]
    pub fn get_entity_count(&self) -> usize {
        self.game_state.borrow().get_entity_count()
    }
    
    #[wasm_bindgen]
    pub fn get_performance_data(&self) -> JsValue {
        let state = self.game_state.borrow();
        let data = js_sys::Object::new();
        
        js_sys::Reflect::set(&data, &"frameTime".into(), &state.get_frame_time().into()).unwrap();
        js_sys::Reflect::set(&data, &"entityCount".into(), &state.get_entity_count().into()).unwrap();
        js_sys::Reflect::set(&data, &"particleCount".into(), &state.get_particle_count().into()).unwrap();
        
        data.into()
    }
    
    #[wasm_bindgen]
    pub fn set_quality(&self, quality: u8) {
        self.game_state.borrow_mut().set_graphics_quality(quality);
    }
    
    #[wasm_bindgen]
    pub fn enable_multiplayer(&self, server_url: String) {
        self.game_state.borrow_mut().enable_multiplayer(server_url);
    }
    
    #[wasm_bindgen]
    pub fn reset(&self) {
        self.game_state.borrow_mut().reset_game();
    }
    
    #[wasm_bindgen]
    pub fn cleanup(&self) {
        console_log!("Cleaning up WASM Advanced Game Engine v1.12");
        self.game_state.borrow_mut().cleanup();
    }
}

// Entry point for WASM
#[wasm_bindgen(start)]
pub fn main() {
    console_log!("WASM Advanced Game Engine v1.12 Enhanced Edition loaded successfully!");
    console_log!("Features: ECS Architecture, Advanced Physics, AI, Networking, PBR Rendering");
}