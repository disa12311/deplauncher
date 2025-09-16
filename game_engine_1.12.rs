use wasm_bindgen::prelude::*;
use js_sys::*;
use web_sys::*;
use std::collections::{HashMap, VecDeque};
use nalgebra::{Vector3, Matrix4, Quaternion, UnitQuaternion};
use serde::{Serialize, Deserialize};

// Advanced Game Engine for Deplauncher 1.12 - Enhanced Edition (Rust)
// Cleaned and optimized version with better architecture and performance

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
    
    #[wasm_bindgen(js_namespace = performance)]
    fn now() -> f64;
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

// === CONSTANTS ===
const MAX_ENTITIES: usize = 5000;
const MAX_PARTICLES: usize = 10000;
const MAX_LIGHTS: usize = 50;
const CANVAS_WIDTH: f32 = 1920.0;
const CANVAS_HEIGHT: f32 = 1080.0;
const PHYSICS_SUBSTEPS: usize = 4;

// === CORE COMPONENTS ===

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Transform {
    pub position: Vector3<f32>,
    pub rotation: UnitQuaternion<f32>,
    pub scale: Vector3<f32>,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vector3::zeros(),
            rotation: UnitQuaternion::identity(),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }
}

impl Transform {
    pub fn new(position: Vector3<f32>) -> Self {
        Self {
            position,
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
    
    pub fn translate(&mut self, delta: Vector3<f32>) {
        self.position += delta;
    }
}

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
}

impl Default for Physics {
    fn default() -> Self {
        Self {
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
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Renderer {
    pub mesh_id: u32,
    pub material_id: u32,
    pub color: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub emission: [f32; 3],
    pub cast_shadows: bool,
    pub receive_shadows: bool,
    pub visible: bool,
}

impl Default for Renderer {
    fn default() -> Self {
        Self {
            mesh_id: 0,
            material_id: 0,
            color: [1.0, 1.0, 1.0, 1.0],
            metallic: 0.0,
            roughness: 0.5,
            emission: [0.0, 0.0, 0.0],
            cast_shadows: true,
            receive_shadows: true,
            visible: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
        Self {
            current: 100.0,
            max: 100.0,
            regeneration: 0.0,
            armor: 0.0,
            invulnerable: false,
            last_damage_time: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AI {
    pub behavior_state: AIState,
    pub target_entity: Option<u32>,
    pub target_position: Option<Vector3<f32>>,
    pub path: VecDeque<Vector3<f32>>,
    pub decision_timer: f32,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AIState {
    Idle,
    Wander,
    Seek,
    Flee,
    Follow,
}

impl Default for AI {
    fn default() -> Self {
        Self {
            behavior_state: AIState::Idle,
            target_entity: None,
            target_position: None,
            path: VecDeque::new(),
            decision_timer: 0.0,
            enabled: true,
        }
    }
}

// === ENTITY SYSTEM ===

#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: u32,
    pub name: String,
    pub tag: String,
    pub active: bool,
    
    // Core components
    pub transform: Transform,
    pub physics: Option<Physics>,
    pub renderer: Option<Renderer>,
    pub health: Option<Health>,
    pub ai: Option<AI>,
}

impl Entity {
    pub fn new(id: u32, name: String, position: Vector3<f32>) -> Self {
        Self {
            id,
            name,
            tag: "Untagged".to_string(),
            active: true,
            transform: Transform::new(position),
            physics: None,
            renderer: None,
            health: None,
            ai: None,
        }
    }
    
    // Component management
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
    
    // Utility methods
    pub fn has_component<T>(&self) -> bool {
        // This would be implemented with proper reflection in a full ECS
        true
    }
    
    pub fn is_alive(&self) -> bool {
        self.active && self.health.as_ref().map_or(true, |h| h.current > 0.0)
    }
}

// === PARTICLE SYSTEM ===

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
}

#[derive(Debug)]
pub struct ParticleSystem {
    particles: Vec<Particle>,
    gravity: Vector3<f32>,
    wind: Vector3<f32>,
}

impl ParticleSystem {
    pub fn new() -> Self {
        Self {
            particles: Vec::with_capacity(MAX_PARTICLES),
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
            
            // Apply forces
            particle.acceleration += self.gravity + self.wind;
            particle.velocity += particle.acceleration * delta_time;
            particle.position += particle.velocity * delta_time;
            particle.rotation += particle.angular_velocity * delta_time;
            
            // Update life and appearance
            particle.life -= delta_time;
            if particle.life <= 0.0 {
                particle.active = false;
                return false;
            }
            
            let life_ratio = particle.life / particle.max_life;
            particle.color[3] *= life_ratio.powf(0.5); // Fade out
            particle.size *= 0.99; // Shrink
            
            particle.acceleration = Vector3::zeros();
            true
        });
    }
    
    pub fn create_explosion(&mut self, position: Vector3<f32>, intensity: f32) {
        let particle_count = (intensity * 20.0) as usize;
        
        for _ in 0..particle_count.min(50) { // Limit burst size
            if self.particles.len() >= MAX_PARTICLES {
                break;
            }
            
            let angle = Math::random() * 2.0 * std::f64::consts::PI;
            let speed = 100.0 + Math::random() * 200.0 * intensity as f64;
            
            let velocity = Vector3::new(
                (angle.cos() * speed) as f32,
                (angle.sin() * speed) as f32,
                ((Math::random() - 0.5) * speed * 0.5) as f32,
            );
            
            let particle = Particle {
                position,
                velocity,
                acceleration: Vector3::zeros(),
                color: [1.0, 0.8, 0.2, 1.0], // Orange fire
                size: 3.0 + Math::random() as f32 * 4.0,
                rotation: 0.0,
                angular_velocity: (Math::random() as f32 - 0.5) * 10.0,
                life: 1.0 + Math::random() as f32 * 2.0,
                max_life: 2.0,
                active: true,
            };
            
            self.particles.push(particle);
        }
    }
    
    pub fn particle_count(&self) -> usize {
        self.particles.len()
    }
}

// === LIGHTING SYSTEM ===

#[derive(Debug, Clone)]
pub struct Light {
    pub position: Vector3<f32>,
    pub direction: Vector3<f32>,
    pub color: [f32; 3],
    pub intensity: f32,
    pub range: f32,
    pub light_type: LightType,
    pub cast_shadows: bool,
    pub active: bool,
}

#[derive(Debug, Clone)]
pub enum LightType {
    Directional,
    Point,
    Spot { angle: f32 },
}

// === CAMERA SYSTEM ===

#[derive(Debug, Clone)]
pub struct Camera {
    pub transform: Transform,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub aspect_ratio: f32,
    pub target_entity: Option<u32>,
    pub follow_speed: f32,
    pub smoothing: f32,
}

impl Camera {
    pub fn new(position: Vector3<f32>, fov: f32, aspect_ratio: f32) -> Self {
        Self {
            transform: Transform::new(position),
            fov,
            near: 0.1,
            far: 1000.0,
            aspect_ratio,
            target_entity: None,
            follow_speed: 5.0,
            smoothing: 0.1,
        }
    }
    
    pub fn follow_entity(&mut self, target_pos: Vector3<f32>, delta_time: f32) {
        let lerp_factor = 1.0 - (-self.follow_speed * delta_time).exp();
        self.transform.position = self.transform.position.lerp(&target_pos, lerp_factor);
    }
    
    pub fn get_view_matrix(&self) -> Matrix4<f32> {
        let eye = self.transform.position;
        let target = eye + self.transform.forward();
        let up = Vector3::y();
        Matrix4::look_at_rh(&eye, &target, &up)
    }
    
    pub fn get_projection_matrix(&self) -> Matrix4<f32> {
        Matrix4::new_perspective(
            self.aspect_ratio,
            self.fov.to_radians(),
            self.near,
            self.far,
        )
    }
}

// === PERFORMANCE METRICS ===

#[derive(Debug, Default)]
pub struct PerformanceMetrics {
    pub last_frame_time: f64,
    pub fps_counter: i32,
    pub fps_timer: f64,
    pub frame_time_ms: f32,
    pub entity_count: usize,
    pub particle_count: usize,
}

// === MAIN GAME STATE ===

#[wasm_bindgen]
pub struct AdvancedGameState {
    // Entity management
    entities: HashMap<u32, Entity>,
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
    
    // Performance
    performance: PerformanceMetrics,
    
    // Physics
    gravity: Vector3<f32>,
    physics_enabled: bool,
    
    // Input
    input_state: HashMap<u32, bool>,
    
    // Graphics settings
    graphics_quality: u8,
    bloom_enabled: bool,
    shadows_enabled: bool,
    pbr_enabled: bool,
}

#[wasm_bindgen]
impl AdvancedGameState {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_log!("Creating Advanced Game State for v1.12 Enhanced Edition");
        
        let mut game_state = Self {
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
            
            performance: PerformanceMetrics {
                last_frame_time: now(),
                ..Default::default()
            },
            
            gravity: Vector3::new(0.0, -980.0, 0.0),
            physics_enabled: true,
            
            input_state: HashMap::new(),
            
            graphics_quality: 2,
            bloom_enabled: true,
            shadows_enabled: true,
            pbr_enabled: true,
        };
        
        game_state.initialize_scene();
        game_state
    }
    
    fn initialize_scene(&mut self) {
        // Create player entity
        let player_id = self.create_entity("Player".to_string(), Vector3::new(CANVAS_WIDTH / 2.0, CANVAS_HEIGHT / 2.0, 0.0));
        
        if let Some(player) = self.entities.get_mut(&player_id) {
            // Setup player components
            let physics = player.add_physics();
            physics.mass = 1.0;
            physics.use_gravity = false; // Top-down view
            physics.drag = 5.0;
            
            let renderer = player.add_renderer();
            renderer.color = [0.2, 0.8, 1.0, 1.0]; // Cyan
            renderer.metallic = 0.1;
            renderer.roughness = 0.3;
            
            let health = player.add_health(100.0);
            health.regeneration = 5.0;
            
            player.tag = "Player".to_string();
        }
        
        // Generate environment
        self.generate_environment();
        
        // Setup lighting
        self.setup_lighting();
        
        // Set camera target
        self.camera.target_entity = Some(player_id);
        
        console_log!("Scene initialized with {} entities", self.entities.len());
    }
    
    fn generate_environment(&mut self) {
        // Create environment objects
        for i in 0..100 {
            let position = Vector3::new(
                Math::random() as f32 * CANVAS_WIDTH,
                Math::random() as f32 * CANVAS_HEIGHT,
                (Math::random() as f32 - 0.5) * 200.0,
            );
            
            let entity_id = self.create_entity(format!("Environment_{}", i), position);
            
            if let Some(entity) = self.entities.get_mut(&entity_id) {
                // Add physics
                let physics = entity.add_physics();
                physics.mass = 0.5 + Math::random() as f32 * 2.0;
                physics.bounciness = 0.3 + Math::random() as f32 * 0.7;
                physics.friction = 0.1 + Math::random() as f32 * 0.8;
                
                // Add renderer
                let renderer = entity.add_renderer();
                renderer.color = [
                    0.5 + Math::random() as f32 * 0.5,
                    0.5 + Math::random() as f32 * 0.5,
                    0.5 + Math::random() as f32 * 0.5,
                    1.0,
                ];
                renderer.metallic = Math::random() as f32;
                renderer.roughness = 0.2 + Math::random() as f32 * 0.8;
                
                // Some have AI
                if Math::random() < 0.3 {
                    let ai = entity.add_ai();
                    ai.behavior_state = AIState::Wander;
                }
                
                entity.tag = "Environment".to_string();
            }
        }
    }
    
    fn setup_lighting(&mut self) {
        // Main directional light
        let main_light = Light {
            position: Vector3::new(CANVAS_WIDTH / 2.0, CANVAS_HEIGHT / 2.0, 1000.0),
            direction: Vector3::new(0.3, -1.0, 0.3).normalize(),
            color: [1.0, 0.95, 0.8],
            intensity: 3.0,
            range: 0.0,
            light_type: LightType::Directional,
            cast_shadows: true,
            active: true,
        };
        self.lights.push(main_light);
        
        // Add some point lights
        for _ in 0..5 {
            let point_light = Light {
                position: Vector3::new(
                    Math::random() as f32 * CANVAS_WIDTH,
                    Math::random() as f32 * CANVAS_HEIGHT,
                    50.0 + Math::random() as f32 * 200.0,
                ),
                direction: Vector3::zeros(),
                color: [
                    0.5 + Math::random() as f32 * 0.5,
                    0.5 + Math::random() as f32 * 0.5,
                    0.5 + Math::random() as f32 * 0.5,
                ],
                intensity: 1.0 + Math::random() as f32 * 2.0,
                range: 100.0 + Math::random() as f32 * 200.0,
                light_type: LightType::Point,
                cast_shadows: false,
                active: true,
            };
            self.lights.push(point_light);
        }
    }
    
    pub fn create_entity(&mut self, name: String, position: Vector3<f32>) -> u32 {
        let id = self.next_entity_id;
        self.next_entity_id += 1;
        
        let entity = Entity::new(id, name, position);
        self.entities.insert(id, entity);
        
        id
    }
    
    #[wasm_bindgen]
    pub fn update(&mut self, current_time: f64) {
        if self.paused {
            return;
        }
        
        let frame_start = now();
        let mut delta_time = ((current_time - self.performance.last_frame_time) / 1000.0) as f32 * self.time_scale;
        self.performance.last_frame_time = current_time;
        
        // Cap delta time
        if delta_time > 0.033 {
            delta_time = 0.033;
        }
        
        // Update FPS counter
        self.performance.fps_counter += 1;
        self.performance.fps_timer += delta_time as f64;
        if self.performance.fps_timer >= 1.0 {
            console_log!("FPS: {}, Entities: {}, Particles: {}", 
                        self.performance.fps_counter, self.entities.len(), self.particle_system.particle_count());
            self.performance.fps_counter = 0;
            self.performance.fps_timer = 0.0;
        }
        
        // Update all systems
        self.update_physics_system(delta_time);
        self.update_ai_system(delta_time);
        self.update_health_system(delta_time);
        self.particle_system.update(delta_time);
        self.update_camera_system(delta_time);
        
        // Process input
        self.process_input(delta_time);
        
        // Collision detection
        self.collision_detection();
        
        // Cleanup
        self.cleanup_entities();
        
        // Update performance metrics
        let frame_end = now();
        self.performance.frame_time_ms = (frame_end - frame_start) as f32;
        self.performance.entity_count = self.entities.len();
        self.performance.particle_count = self.particle_system.particle_count();
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
                
                if let Some(physics) = &mut entity.physics {
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
                        let drag_force = 0.5 * 1.225 * speed * speed * physics.drag; // air density
                        let drag_acceleration = -(physics.velocity.normalize() * drag_force) / physics.mass;
                        physics.acceleration += drag_acceleration;
                    }
                    
                    // Integration
                    physics.velocity += physics.acceleration * sub_delta;
                    entity.transform.position += physics.velocity * sub_delta;
                    
                    // Apply friction
                    physics.velocity *= 1.0 - (physics.friction * sub_delta);
                    
                    // Reset acceleration
                    physics.acceleration = Vector3::zeros();
                }
            }
        }
    }
    
    fn update_ai_system(&mut self, delta_time: f32) {
        // Collect entity positions for AI decision making
        let entity_positions: HashMap<u32, Vector3<f32>> = self.entities
            .iter()
            .filter(|(_, e)| e.active)
            .map(|(&id, entity)| (id, entity.transform.position))
            .collect();
        
        for entity in self.entities.values_mut() {
            if !entity.active {
                continue;
            }
            
            if let Some(ai) = &mut entity.ai {
                if !ai.enabled {
                    continue;
                }
                
                ai.decision_timer -= delta_time;
                
                if ai.decision_timer <= 0.0 {
                    ai.decision_timer = 0.5; // Decision interval
                    
                    match ai.behavior_state {
                        AIState::Wander => {
                            if ai.path.is_empty() {
                                let target = Vector3::new(
                                    Math::random() as f32 * CANVAS_WIDTH,
                                    Math::random() as f32 * CANVAS_HEIGHT,
                                    entity.transform.position.z,
                                );
                                ai.target_position = Some(target);
                                ai.path.push_back(target);
                            }
                        }
                        AIState::Seek => {
                            // Find closest player
                            if let Some(player_pos) = entity_positions.get(&1) {
                                ai.target_position = Some(*player_pos);
                                ai.target_entity = Some(1);
                            }
                        }
                        _ => {}
                    }
                }
                
                // Execute behavior
                if let Some(target) = ai.target_position {
                    let direction = target - entity.transform.position;
                    let distance = direction.magnitude();
                    
                    if distance > 5.0 {
                        let move_force = direction.normalize() * 100.0;
                        
                        if let Some(physics) = &mut entity.physics {
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
            
            if let Some(health) = &mut entity.health {
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
    
    fn update_camera_system(&mut self, delta_time: f32) {
        if let Some(target_id) = self.camera.target_entity {
            if let Some(target_entity) = self.entities.get(&target_id) {
                self.camera.follow_entity(target_entity.transform.position, delta_time);
            }
        }
    }
    
    fn process_input(&mut self, delta_time: f32) {
        // Find player entity
        if let Some(player) = self.entities.get_mut(&1) {
            if let Some(physics) = &mut player.physics {
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
                
                // Normalize and apply
                if move_direction.magnitude() > 0.0 {
                    move_direction = move_direction.normalize();
                    physics.acceleration += move_direction * move_speed;
                }
            }
        }
    }
    
    fn collision_detection(&mut self) {
        let entity_ids: Vec<u32> = self.entities.keys().cloned().collect();
        
        for i in 0..entity_ids.len() {
            for j in (i + 1)..entity_ids.len() {
                let id_a = entity_ids[i];
                let id_b = entity_ids[j];
                
                // Check collision
                let collision_data = {
                    let entity_a = &self.entities[&id_a];
                    let entity_b = &self.entities[&id_b];
                    
                    if !entity_a.active || !entity_b.active {
                        continue;
                    }
                    
                    let distance = (entity_a.transform.position - entity_b.transform.position).magnitude();
                    let collision_radius = 32.0;
                    
                    if distance < collision_radius {
                        Some((
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
                
                if let Some((pos_a, pos_b, distance, tag_a, tag_b)) = collision_data {
                    // Handle collision response
                    let direction = (pos_a - pos_b).normalize();
                    let overlap = 32.0 - distance;
                    
                    // Separate entities
                    if let Some(entity_a) = self.entities.get_mut(&id_a) {
                        entity_a.transform.position += direction * overlap * 0.5;
                        
                        if let Some(physics_a) = &mut entity_a.physics {
                            physics_a.velocity += direction * physics_a.bounciness * 150.0;
                        }
                    }
                    
                    if let Some(entity_b) = self.entities.get_mut(&id_b) {
                        entity_b.transform.position -= direction * overlap * 0.5;
                        
                        if let Some(physics_b) = &mut entity_b.physics {
                            physics_b.velocity -= direction * physics_b.bounciness * 150.0;
                        }
                    }
                    
                    // Handle specific interactions
                    if tag_a == "Player" || tag_b == "Player" {
                        self.score += 5;
                        
                        // Create collision effect
                        let effect_pos = (pos_a + pos_b) * 0.5;
                        self.particle_system.create_explosion(effect_pos, 0.5);
                    }
                }
            }
        }
    }
    
    fn cleanup_entities(&mut self) {
        self.entities.retain(|_, entity| entity.active && entity.is_alive());
    }
    
    // === WASM EXPORTS ===
    
    #[wasm_bindgen]
    pub fn handle_key_event(&mut self, key_code: u32, pressed: bool) {
        self.input_state.insert(key_code, pressed);
        
        if pressed && key_code == 32 { // Space
            self.paused = !self.paused;
        }
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
        self.particle_system.particle_count()
    }
    
    #[wasm_bindgen]
    pub fn get_frame_time(&self) -> f32 {
        self.performance.frame_time_ms
    }
    
    #[wasm_bindgen]
    pub fn set_graphics_quality(&mut self, quality: u8) {
        self.graphics_quality = quality;
        
        match quality {
            0 => { // Low
                self.shadows_enabled = false;
                self.bloom_enabled = false;
                self.pbr_enabled = false;
            }
            1 => { // Medium
                self.shadows_enabled = true;
                self.bloom_enabled = true;
                self.pbr_enabled = true;
            }
            2 => { // High
                self.shadows_enabled = true;
                self.bloom_enabled = true;
                self.pbr_enabled = true;
            }
            _ => {}
        }
        
        console_log!("Graphics quality set to {}", quality);
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
        self.particle_system = ParticleSystem::new();
        self.lights.clear();
    }
}

// === ENTRY POINT ===

#[wasm_bindgen(start)]
pub fn main() {
    console_log!("WASM Advanced Game Engine v1.12 Enhanced Edition loaded successfully!");
    console_log!("Features: ECS Architecture, Advanced Physics, AI, PBR Rendering");
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
}
