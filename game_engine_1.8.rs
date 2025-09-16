use wasm_bindgen::prelude::*;
use js_sys::*;
use web_sys::*;
use std::collections::HashMap;

// Game Engine for Deplauncher 1.8 - Classic Edition (Rust)
// Cleaned and optimized for memory safety and performance

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
const MAX_ENTITIES: usize = 1000;
const CANVAS_WIDTH: f32 = 800.0;
const CANVAS_HEIGHT: f32 = 600.0;
const COLLISION_RADIUS: f32 = 32.0;

// === CORE STRUCTURES ===

#[derive(Debug, Clone, Copy)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Vector2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
    
    pub fn magnitude(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
    
    pub fn normalized(&self) -> Self {
        let mag = self.magnitude();
        if mag > 0.001 {
            Self::new(self.x / mag, self.y / mag)
        } else {
            Self::zero()
        }
    }
    
    pub fn distance_to(&self, other: &Vector2) -> f32 {
        (*self - *other).magnitude()
    }
}

impl std::ops::Add for Vector2 {
    type Output = Self;
    
    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y)
    }
}

impl std::ops::AddAssign for Vector2 {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl std::ops::Sub for Vector2 {
    type Output = Self;
    
    fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y)
    }
}

impl std::ops::Mul<f32> for Vector2 {
    type Output = Self;
    
    fn mul(self, scalar: f32) -> Self {
        Self::new(self.x * scalar, self.y * scalar)
    }
}

impl std::ops::MulAssign<f32> for Vector2 {
    fn mul_assign(&mut self, scalar: f32) {
        self.x *= scalar;
        self.y *= scalar;
    }
}

// === ENTITY SYSTEM ===

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct Entity {
    pub position: Vector2,
    pub velocity: Vector2,
    pub rotation: f32,
    pub texture_id: u32,
    pub active: bool,
    pub health: i32,
    pub name: String,
    pub entity_type: EntityType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EntityType {
    Player,
    Environment,
    Projectile,
    Pickup,
}

impl Entity {
    pub fn new(position: Vector2, texture_id: u32, name: String, entity_type: EntityType) -> Self {
        Self {
            position,
            velocity: Vector2::zero(),
            rotation: 0.0,
            texture_id,
            active: true,
            health: 100,
            name,
            entity_type,
        }
    }
    
    pub fn update(&mut self, delta_time: f32) {
        if !self.active {
            return;
        }
        
        // Apply velocity
        self.position += self.velocity * delta_time;
        
        // Apply rotation
        self.rotation += delta_time * 45.0; // 45 degrees per second
        if self.rotation > 360.0 {
            self.rotation -= 360.0;
        }
        
        // Boundary wrapping for classic arcade feel
        if self.position.x < 0.0 {
            self.position.x = CANVAS_WIDTH;
        } else if self.position.x > CANVAS_WIDTH {
            self.position.x = 0.0;
        }
        
        if self.position.y < 0.0 {
            self.position.y = CANVAS_HEIGHT;
        } else if self.position.y > CANVAS_HEIGHT {
            self.position.y = 0.0;
        }
        
        // Apply friction for classic physics
        self.velocity *= 0.95;
    }
    
    pub fn apply_force(&mut self, force: Vector2) {
        self.velocity += force;
    }
    
    pub fn is_alive(&self) -> bool {
        self.active && self.health > 0
    }
}

// === PARTICLE SYSTEM ===

#[derive(Debug, Clone)]
pub struct Particle {
    pub position: Vector2,
    pub velocity: Vector2,
    pub life: f32,
    pub max_life: f32,
    pub color: [u8; 3],
    pub active: bool,
}

impl Particle {
    pub fn new(position: Vector2, velocity: Vector2, life: f32, color: [u8; 3]) -> Self {
        Self {
            position,
            velocity,
            life,
            max_life: life,
            color,
            active: true,
        }
    }
    
    pub fn update(&mut self, delta_time: f32) {
        if !self.active {
            return;
        }
        
        // Apply gravity
        self.velocity.y += 98.0 * delta_time;
        
        // Update position
        self.position += self.velocity * delta_time;
        
        // Update life
        self.life -= delta_time;
        if self.life <= 0.0 {
            self.active = false;
        }
    }
    
    pub fn life_ratio(&self) -> f32 {
        if self.max_life > 0.0 {
            (self.life / self.max_life).max(0.0).min(1.0)
        } else {
            0.0
        }
    }
}

#[derive(Debug)]
pub struct ParticleSystem {
    particles: Vec<Particle>,
    max_particles: usize,
}

impl ParticleSystem {
    pub fn new(max_particles: usize) -> Self {
        Self {
            particles: Vec::with_capacity(max_particles),
            max_particles,
        }
    }
    
    pub fn update(&mut self, delta_time: f32) {
        // Update existing particles
        for particle in &mut self.particles {
            particle.update(delta_time);
        }
        
        // Remove dead particles
        self.particles.retain(|p| p.active);
    }
    
    pub fn create_explosion(&mut self, position: Vector2, count: usize) {
        let actual_count = count.min(20).min(self.max_particles - self.particles.len());
        
        for _ in 0..actual_count {
            let angle = Math::random() * 2.0 * std::f64::consts::PI;
            let speed = 50.0 + Math::random() * 100.0;
            
            let velocity = Vector2::new(
                (angle.cos() * speed) as f32,
                (angle.sin() * speed) as f32,
            );
            
            let life = 0.5 + Math::random() as f32 * 1.5;
            let color = [255, 150, 50]; // Orange explosion
            
            let particle = Particle::new(position, velocity, life, color);
            self.particles.push(particle);
        }
    }
    
    pub fn particle_count(&self) -> usize {
        self.particles.len()
    }
    
    pub fn get_particles(&self) -> &[Particle] {
        &self.particles
    }
    
    pub fn clear(&mut self) {
        self.particles.clear();
    }
}

// === PERFORMANCE METRICS ===

#[derive(Debug, Default)]
pub struct PerformanceMetrics {
    pub last_frame_time: f64,
    pub fps_counter: i32,
    pub fps_timer: f64,
    pub frame_time_ms: f32,
}

// === AI SYSTEM ===

#[derive(Debug)]
pub struct AISystem {
    decision_timer: f32,
}

impl AISystem {
    pub fn new() -> Self {
        Self {
            decision_timer: 0.0,
        }
    }
    
    pub fn update(&mut self, entities: &mut [Entity], delta_time: f32) {
        self.decision_timer += delta_time;
        
        let current_time = now() * 0.001; // Convert to seconds
        
        for (i, entity) in entities.iter_mut().enumerate() {
            if !entity.active || entity.entity_type == EntityType::Player {
                continue;
            }
            
            match entity.entity_type {
                EntityType::Environment => {
                    // Simple movement pattern with time offset
                    let time_offset = i as f64 * 0.5;
                    let move_speed = 50.0;
                    
                    entity.velocity.x = (current_time + time_offset).sin() as f32 * move_speed;
                    entity.velocity.y = (current_time + time_offset).cos() as f32 * move_speed;
                    
                    // Occasional random impulses
                    if self.decision_timer > 2.0 && Math::random() < 0.1 {
                        let impulse = Vector2::new(
                            (Math::random() as f32 - 0.5) * 200.0,
                            (Math::random() as f32 - 0.5) * 200.0,
                        );
                        entity.apply_force(impulse);
                    }
                }
                _ => {}
            }
        }
        
        // Reset decision timer
        if self.decision_timer > 2.0 {
            self.decision_timer = 0.0;
        }
    }
}

// === COLLISION SYSTEM ===

#[derive(Debug)]
pub struct CollisionSystem;

impl CollisionSystem {
    pub fn new() -> Self {
        Self
    }
    
    pub fn check_collisions(&self, entities: &mut [Entity], particle_system: &mut ParticleSystem) -> i32 {
        let mut score_increment = 0;
        
        let entity_count = entities.len();
        for i in 0..entity_count {
            for j in (i + 1)..entity_count {
                let (left, right) = entities.split_at_mut(j);
                let entity_a = &mut left[i];
                let entity_b = &mut right[0];
                
                if !entity_a.active || !entity_b.active {
                    continue;
                }
                
                let distance = entity_a.position.distance_to(&entity_b.position);
                
                if distance < COLLISION_RADIUS {
                    // Handle collision
                    self.handle_collision(entity_a, entity_b, particle_system);
                    
                    // Update score if player is involved
                    if entity_a.entity_type == EntityType::Player || entity_b.entity_type == EntityType::Player {
                        score_increment += 10;
                    }
                }
            }
        }
        
        score_increment
    }
    
    fn handle_collision(&self, entity_a: &mut Entity, entity_b: &mut Entity, particle_system: &mut ParticleSystem) {
        // Calculate collision response
        let direction = (entity_a.position - entity_b.position).normalized();
        let bounce_force = 100.0;
        
        // Apply bounce
        entity_a.apply_force(direction * bounce_force);
        entity_b.apply_force(direction * -bounce_force);
        
        // Separate entities to prevent overlap
        let overlap = COLLISION_RADIUS - entity_a.position.distance_to(&entity_b.position);
        if overlap > 0.0 {
            let separation = direction * (overlap * 0.5);
            entity_a.position += separation;
            entity_b.position -= separation;
        }
        
        // Create particle effect at collision point
        let collision_point = (entity_a.position + entity_b.position) * 0.5;
        particle_system.create_explosion(collision_point, 3);
    }
}

// === INPUT SYSTEM ===

#[derive(Debug)]
pub struct InputSystem {
    keys: HashMap<u32, bool>,
}

impl InputSystem {
    pub fn new() -> Self {
        Self {
            keys: HashMap::new(),
        }
    }
    
    pub fn set_key(&mut self, key_code: u32, pressed: bool) {
        self.keys.insert(key_code, pressed);
    }
    
    pub fn is_key_pressed(&self, key_code: u32) -> bool {
        *self.keys.get(&key_code).unwrap_or(&false)
    }
    
    pub fn update_player(&self, player: &mut Entity, delta_time: f32) {
        let move_speed = 200.0;
        let mut acceleration = Vector2::zero();
        
        // WASD and Arrow key movement
        if self.is_key_pressed(87) || self.is_key_pressed(38) { // W or Up
            acceleration.y -= move_speed;
        }
        if self.is_key_pressed(83) || self.is_key_pressed(40) { // S or Down
            acceleration.y += move_speed;
        }
        if self.is_key_pressed(65) || self.is_key_pressed(37) { // A or Left
            acceleration.x -= move_speed;
        }
        if self.is_key_pressed(68) || self.is_key_pressed(39) { // D or Right
            acceleration.x += move_speed;
        }
        
        // Normalize diagonal movement and apply
        if acceleration.magnitude() > 0.0 {
            acceleration = acceleration.normalized() * move_speed;
            player.velocity += acceleration * delta_time;
        }
    }
}

// === MAIN GAME STATE ===

#[wasm_bindgen]
pub struct GameState {
    entities: Vec<Entity>,
    particle_system: ParticleSystem,
    ai_system: AISystem,
    collision_system: CollisionSystem,
    input_system: InputSystem,
    
    // Camera
    camera_x: f32,
    camera_y: f32,
    
    // Game state
    score: i32,
    level: i32,
    paused: bool,
    
    // Performance
    performance: PerformanceMetrics,
}

#[wasm_bindgen]
impl GameState {
    #[wasm_bindgen(constructor)]
    pub fn new() -> GameState {
        console_log!("Creating new GameState for v1.8 Classic Edition");
        
        let mut game_state = GameState {
            entities: Vec::with_capacity(MAX_ENTITIES),
            particle_system: ParticleSystem::new(MAX_ENTITIES / 4),
            ai_system: AISystem::new(),
            collision_system: CollisionSystem::new(),
            input_system: InputSystem::new(),
            
            camera_x: CANVAS_WIDTH / 2.0,
            camera_y: CANVAS_HEIGHT / 2.0,
            
            score: 0,
            level: 1,
            paused: false,
            
            performance: PerformanceMetrics {
                last_frame_time: now(),
                ..Default::default()
            },
        };
        
        game_state.initialize_entities();
        game_state
    }
    
    fn initialize_entities(&mut self) {
        // Create player entity
        let player = Entity::new(
            Vector2::new(CANVAS_WIDTH / 2.0, CANVAS_HEIGHT / 2.0),
            0,
            "Player".to_string(),
            EntityType::Player,
        );
        self.entities.push(player);
        
        // Create environment entities
        for i in 0..20 {
            let position = Vector2::new(
                50.0 + (CANVAS_WIDTH - 100.0) * Math::random() as f32,
                50.0 + (CANVAS_HEIGHT - 100.0) * Math::random() as f32,
            );
            
            let texture_id = 1 + (Math::random() * 3.0) as u32;
            let name = format!("Object_{}", i);
            
            let mut env_entity = Entity::new(position, texture_id, name, EntityType::Environment);
            
            // Give some initial velocity for dynamic gameplay
            env_entity.velocity = Vector2::new(
                (Math::random() as f32 - 0.5) * 50.0,
                (Math::random() as f32 - 0.5) * 50.0,
            );
            
            self.entities.push(env_entity);
        }
        
        console_log!("Initialized {} entities", self.entities.len());
    }
    
    #[wasm_bindgen]
    pub fn update(&mut self, current_time: f64) {
        if self.paused {
            return;
        }
        
        let frame_start = now();
        let mut delta_time = ((current_time - self.performance.last_frame_time) / 1000.0) as f32;
        self.performance.last_frame_time = current_time;
        
        // Cap delta time to prevent large jumps
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
        
        // Update player input
        if let Some(player) = self.entities.first_mut() {
            if player.entity_type == EntityType::Player {
                self.input_system.update_player(player, delta_time);
            }
        }
        
        // Update all entities
        for entity in &mut self.entities {
            entity.update(delta_time);
        }
        
        // Update AI system
        self.ai_system.update(&mut self.entities, delta_time);
        
        // Handle collisions
        let score_increment = self.collision_system.check_collisions(&mut self.entities, &mut self.particle_system);
        self.score += score_increment;
        
        // Update particle system
        self.particle_system.update(delta_time);
        
        // Update camera to follow player
        if let Some(player) = self.entities.first() {
            if player.entity_type == EntityType::Player {
                let lerp_speed = 3.0 * delta_time;
                self.camera_x += (player.position.x - self.camera_x) * lerp_speed;
                self.camera_y += (player.position.y - self.camera_y) * lerp_speed;
            }
        }
        
        // Cleanup inactive entities
        self.entities.retain(|e| e.is_alive());
        
        // Calculate frame time
        let frame_end = now();
        self.performance.frame_time_ms = (frame_end - frame_start) as f32;
    }
    
    #[wasm_bindgen]
    pub fn handle_key_event(&mut self, key_code: u32, pressed: bool) {
        self.input_system.set_key(key_code, pressed);
        
        // Handle special keys
        if pressed {
            match key_code {
                32 => { // Space bar
                    self.paused = !self.paused;
                    console_log!("Game {}", if self.paused { "paused" } else { "resumed" });
                }
                82 => { // R key
                    self.reset_game();
                }
                _ => {}
            }
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
    pub fn is_paused(&self) -> bool {
        self.paused
    }
    
    #[wasm_bindgen]
    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }
    
    #[wasm_bindgen]
    pub fn get_camera_x(&self) -> f32 {
        self.camera_x
    }
    
    #[wasm_bindgen]
    pub fn get_camera_y(&self) -> f32 {
        self.camera_y
    }
    
    #[wasm_bindgen]
    pub fn get_entity_positions(&self) -> Vec<f32> {
        let mut positions = Vec::with_capacity(self.entities.len() * 3);
        
        for entity in &self.entities {
            if entity.active {
                positions.push(entity.position.x);
                positions.push(entity.position.y);
                positions.push(entity.rotation);
            }
        }
        
        positions
    }
    
    #[wasm_bindgen]
    pub fn get_particle_positions(&self) -> Vec<f32> {
        let mut positions = Vec::with_capacity(self.particle_system.particle_count() * 3);
        
        for particle in self.particle_system.get_particles() {
            if particle.active {
                positions.push(particle.position.x);
                positions.push(particle.position.y);
                positions.push(particle.life_ratio());
            }
        }
        
        positions
    }
    
    #[wasm_bindgen]
    pub fn add_entity(&mut self, x: f32, y: f32, texture_id: u32, name: String) -> bool {
        if self.entities.len() < MAX_ENTITIES {
            let entity = Entity::new(
                Vector2::new(x, y),
                texture_id,
                name,
                EntityType::Environment,
            );
            self.entities.push(entity);
            true
        } else {
            false
        }
    }
    
    #[wasm_bindgen]
    pub fn reset_game(&mut self) {
        console_log!("Resetting game state");
        
        self.entities.clear();
        self.particle_system.clear();
        self.score = 0;
        self.level = 1;
        self.paused = false;
        
        self.initialize_entities();
        
        // Reset camera
        self.camera_x = CANVAS_WIDTH / 2.0;
        self.camera_y = CANVAS_HEIGHT / 2.0;
    }
    
    #[wasm_bindgen]
    pub fn get_frame_time(&self) -> f32 {
        self.performance.frame_time_ms
    }
}

// === WASM ENGINE WRAPPER ===

#[wasm_bindgen]
pub struct WasmGameEngine {
    game_state: GameState,
}

#[wasm_bindgen]
impl WasmGameEngine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmGameEngine {
        console_log!("Initializing WASM Game Engine v1.8 Classic");
        
        // Set panic hook for better error messages
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        
        WasmGameEngine {
            game_state: GameState::new(),
        }
    }
    
    #[wasm_bindgen]
    pub fn update_frame(&mut self, current_time: f64) {
        self.game_state.update(current_time);
    }
    
    #[wasm_bindgen]
    pub fn handle_key(&mut self, key_code: u32, pressed: bool) {
        self.game_state.handle_key_event(key_code, pressed);
    }
    
    #[wasm_bindgen]
    pub fn get_score(&self) -> i32 {
        self.game_state.get_score()
    }
    
    #[wasm_bindgen]
    pub fn get_entity_count(&self) -> usize {
        self.game_state.get_entity_count()
    }
    
    #[wasm_bindgen]
    pub fn get_particle_count(&self) -> usize {
        self.game_state.get_particle_count()
    }
    
    #[wasm_bindgen]
    pub fn get_positions(&self) -> Vec<f32> {
        self.game_state.get_entity_positions()
    }
    
    #[wasm_bindgen]
    pub fn get_particle_data(&self) -> Vec<f32> {
        self.game_state.get_particle_positions()
    }
    
    #[wasm_bindgen]
    pub fn get_camera_position(&self) -> Vec<f32> {
        vec![self.game_state.get_camera_x(), self.game_state.get_camera_y()]
    }
    
    #[wasm_bindgen]
    pub fn is_paused(&self) -> bool {
        self.game_state.is_paused()
    }
    
    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.game_state.reset_game();
    }
    
    #[wasm_bindgen]
    pub fn get_performance_info(&self) -> JsValue {
        let info = js_sys::Object::new();
        
        js_sys::Reflect::set(&info, &"frameTime".into(), &self.game_state.get_frame_time().into()).unwrap();
        js_sys::Reflect::set(&info, &"entityCount".into(), &self.game_state.get_entity_count().into()).unwrap();
        js_sys::Reflect::set(&info, &"particleCount".into(), &self.game_state.get_particle_count().into()).unwrap();
        js_sys::Reflect::set(&info, &"score".into(), &self.game_state.get_score().into()).unwrap();
        
        info.into()
    }
    
    #[wasm_bindgen]
    pub fn cleanup(&self) {
        console_log!("Cleaning up WASM Game Engine v1.8");
        // Rust's ownership system handles cleanup automatically
    }
}

// === ENTRY POINT ===

#[wasm_bindgen(start)]
pub fn main() {
    console_log!("WASM Game Engine v1.8 Classic Edition loaded successfully!");
    console_log!("Features: Memory-safe Rust, Optimized Physics, Particle Effects");
}
