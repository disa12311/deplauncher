use wasm_bindgen::prelude::*;
use js_sys::*;
use web_sys::*;
use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

// Game Engine for Deplauncher 1.8 - Classic Edition (Rust)
// Memory-safe implementation with WASM GC integration

const MAX_ENTITIES: usize = 1000;
const CANVAS_WIDTH: f32 = 800.0;
const CANVAS_HEIGHT: f32 = 600.0;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
    
    fn alert(s: &str);
    
    #[wasm_bindgen(js_namespace = performance)]
    fn now() -> f64;
}

// Logging macro
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct Entity {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub rotation: f32,
    pub texture_id: u32,
    pub active: bool,
    pub health: i32,
    pub name: String,
    pub entity_type: String,
}

impl Entity {
    fn new(x: f32, y: f32, texture_id: u32, name: String) -> Self {
        Entity {
            x,
            y,
            z: 0.0,
            velocity_x: 0.0,
            velocity_y: 0.0,
            rotation: 0.0,
            texture_id,
            active: true,
            health: 100,
            name,
            entity_type: "default".to_string(),
        }
    }
    
    fn update(&mut self, delta_time: f32) {
        // Apply velocity
        self.x += self.velocity_x * delta_time;
        self.y += self.velocity_y * delta_time;
        
        // Apply rotation
        self.rotation += delta_time * 45.0; // 45 degrees per second
        if self.rotation > 360.0 {
            self.rotation -= 360.0;
        }
        
        // Boundary wrapping for classic arcade feel
        if self.x < 0.0 {
            self.x = CANVAS_WIDTH;
        }
        if self.x > CANVAS_WIDTH {
            self.x = 0.0;
        }
        if self.y < 0.0 {
            self.y = CANVAS_HEIGHT;
        }
        if self.y > CANVAS_HEIGHT {
            self.y = 0.0;
        }
        
        // Apply friction for classic physics
        self.velocity_x *= 0.95;
        self.velocity_y *= 0.95;
    }
}

#[wasm_bindgen]
pub struct GameState {
    entities: Vec<Entity>,
    camera_x: f32,
    camera_y: f32,
    score: i32,
    paused: bool,
    last_frame_time: f64,
    fps_counter: i32,
    fps_timer: f64,
    input_state: HashMap<u32, bool>,
    particle_system: ParticleSystem,
}

#[wasm_bindgen]
impl GameState {
    #[wasm_bindgen(constructor)]
    pub fn new() -> GameState {
        console_log!("Creating new GameState for v1.8 Classic Edition");
        
        let mut game_state = GameState {
            entities: Vec::with_capacity(MAX_ENTITIES),
            camera_x: CANVAS_WIDTH / 2.0,
            camera_y: CANVAS_HEIGHT / 2.0,
            score: 0,
            paused: false,
            last_frame_time: now(),
            fps_counter: 0,
            fps_timer: 0.0,
            input_state: HashMap::new(),
            particle_system: ParticleSystem::new(),
        };
        
        // Initialize with default entities
        game_state.initialize_entities();
        game_state
    }
    
    fn initialize_entities(&mut self) {
        // Create player entity
        let player = Entity::new(
            CANVAS_WIDTH / 2.0,
            CANVAS_HEIGHT / 2.0,
            0,
            "Player".to_string(),
        );
        self.entities.push(player);
        
        // Create environment entities
        for i in 0..10 {
            let x = js_sys::Math::random() as f32 * CANVAS_WIDTH;
            let y = js_sys::Math::random() as f32 * CANVAS_HEIGHT;
            let texture_id = 1 + (js_sys::Math::random() * 3.0) as u32;
            
            let mut env_entity = Entity::new(x, y, texture_id, format!("Object_{}", i));
            env_entity.entity_type = "environment".to_string();
            self.entities.push(env_entity);
        }
        
        console_log!("Initialized {} entities", self.entities.len());
    }
    
    #[wasm_bindgen]
    pub fn update(&mut self, current_time: f64) {
        if self.paused {
            return;
        }
        
        let delta_time = ((current_time - self.last_frame_time) / 1000.0) as f32;
        self.last_frame_time = current_time;
        
        // Update FPS counter
        self.fps_counter += 1;
        self.fps_timer += delta_time as f64;
        if self.fps_timer >= 1.0 {
            console_log!("FPS: {}", self.fps_counter);
            self.fps_counter = 0;
            self.fps_timer = 0.0;
        }
        
        // Update all entities
        for entity in &mut self.entities {
            if entity.active {
                entity.update(delta_time);
                
                // Simple AI for non-player entities
                if entity.name != "Player" && entity.entity_type == "environment" {
                    let time_factor = current_time * 0.001;
                    entity.velocity_x = (time_factor.sin() as f32) * 50.0;
                    entity.velocity_y = (time_factor.cos() as f32) * 50.0;
                }
            }
        }
        
        // Handle input
        self.handle_input(delta_time);
        
        // Perform collision detection
        self.collision_detection();
        
        // Update particle system
        self.particle_system.update(delta_time);
        
        // Update camera to follow player
        if let Some(player) = self.entities.first() {
            self.camera_x = player.x;
            self.camera_y = player.y;
        }
        
        // Cleanup inactive entities
        self.entities.retain(|e| e.active);
    }
    
    fn handle_input(&mut self, delta_time: f32) {
        if let Some(player) = self.entities.first_mut() {
            let move_speed = 200.0 * delta_time;
            
            // W key or Up arrow
            if self.input_state.get(&87).unwrap_or(&false) || 
               self.input_state.get(&38).unwrap_or(&false) {
                player.velocity_y = -move_speed / delta_time;
            }
            
            // S key or Down arrow
            if self.input_state.get(&83).unwrap_or(&false) || 
               self.input_state.get(&40).unwrap_or(&false) {
                player.velocity_y = move_speed / delta_time;
            }
            
            // A key or Left arrow
            if self.input_state.get(&65).unwrap_or(&false) || 
               self.input_state.get(&37).unwrap_or(&false) {
                player.velocity_x = -move_speed / delta_time;
            }
            
            // D key or Right arrow
            if self.input_state.get(&68).unwrap_or(&false) || 
               self.input_state.get(&39).unwrap_or(&false) {
                player.velocity_x = move_speed / delta_time;
            }
        }
    }
    
    fn collision_detection(&mut self) {
        let entities_len = self.entities.len();
        
        for i in 0..entities_len {
            for j in (i + 1)..entities_len {
                let (left, right) = self.entities.split_at_mut(j);
                let entity_a = &mut left[i];
                let entity_b = &mut right[0];
                
                if !entity_a.active || !entity_b.active {
                    continue;
                }
                
                let dx = entity_a.x - entity_b.x;
                let dy = entity_a.y - entity_b.y;
                let distance = (dx * dx + dy * dy).sqrt();
                
                if distance < 32.0 {
                    // Collision detected
                    if entity_a.name == "Player" {
                        self.score += 10;
                        
                        // Create particle effect
                        self.particle_system.create_explosion(entity_b.x, entity_b.y, 5);
                    }
                    
                    // Bounce effect
                    let bounce_force = 100.0;
                    let normalized_dx = dx / distance;
                    let normalized_dy = dy / distance;
                    
                    entity_a.velocity_x += normalized_dx * bounce_force;
                    entity_a.velocity_y += normalized_dy * bounce_force;
                    entity_b.velocity_x -= normalized_dx * bounce_force;
                    entity_b.velocity_y -= normalized_dy * bounce_force;
                }
            }
        }
    }
    
    #[wasm_bindgen]
    pub fn handle_key_event(&mut self, key_code: u32, pressed: bool) {
        self.input_state.insert(key_code, pressed);
        
        // Handle special keys
        if pressed && key_code == 32 { // Space bar
            self.paused = !self.paused;
            console_log!("Game {}", if self.paused { "paused" } else { "resumed" });
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
            positions.push(entity.x);
            positions.push(entity.y);
            positions.push(entity.rotation);
        }
        
        positions
    }
    
    #[wasm_bindgen]
    pub fn add_entity(&mut self, x: f32, y: f32, texture_id: u32, name: String) -> bool {
        if self.entities.len() < MAX_ENTITIES {
            let entity = Entity::new(x, y, texture_id, name);
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
        self.score = 0;
        self.paused = false;
        self.particle_system = ParticleSystem::new();
        self.initialize_entities();
    }
}

// Particle system for visual effects
#[derive(Debug)]
struct Particle {
    x: f32,
    y: f32,
    velocity_x: f32,
    velocity_y: f32,
    life: f32,
    max_life: f32,
    color: (u8, u8, u8),
}

#[derive(Debug)]
struct ParticleSystem {
    particles: Vec<Particle>,
}

impl ParticleSystem {
    fn new() -> Self {
        ParticleSystem {
            particles: Vec::new(),
        }
    }
    
    fn update(&mut self, delta_time: f32) {
        // Update existing particles
        for particle in &mut self.particles {
            particle.x += particle.velocity_x * delta_time;
            particle.y += particle.velocity_y * delta_time;
            particle.life -= delta_time;
            
            // Apply gravity
            particle.velocity_y += 98.0 * delta_time; // gravity
        }
        
        // Remove dead particles
        self.particles.retain(|p| p.life > 0.0);
    }
    
    fn create_explosion(&mut self, x: f32, y: f32, count: usize) {
        for _ in 0..count {
            let angle = js_sys::Math::random() * 2.0 * std::f64::consts::PI;
            let speed = 50.0 + js_sys::Math::random() * 100.0;
            
            let particle = Particle {
                x,
                y,
                velocity_x: (angle.cos() * speed) as f32,
                velocity_y: (angle.sin() * speed) as f32,
                life: 1.0,
                max_life: 1.0,
                color: (255, 100, 0), // Orange explosion
            };
            
            self.particles.push(particle);
        }
    }
}

// WASM GC utilities and memory management
#[wasm_bindgen]
pub struct WasmGameEngine {
    game_state: Rc<RefCell<GameState>>,
}

#[wasm_bindgen]
impl WasmGameEngine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmGameEngine {
        console_log!("Initializing WASM Game Engine v1.8");
        
        // Set panic hook for better error messages
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        
        WasmGameEngine {
            game_state: Rc::new(RefCell::new(GameState::new())),
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
    pub fn get_score(&self) -> i32 {
        self.game_state.borrow().get_score()
    }
    
    #[wasm_bindgen]
    pub fn get_entity_count(&self) -> usize {
        self.game_state.borrow().get_entity_count()
    }
    
    #[wasm_bindgen]
    pub fn get_positions(&self) -> Vec<f32> {
        self.game_state.borrow().get_entity_positions()
    }
    
    #[wasm_bindgen]
    pub fn reset(&self) {
        self.game_state.borrow_mut().reset_game();
    }
    
    #[wasm_bindgen]
    pub fn cleanup(&self) {
        console_log!("Cleaning up WASM Game Engine v1.8");
        // Rust's ownership system handles cleanup automatically
        // This method is provided for explicit cleanup if needed
    }
}

// Entry point for WASM
#[wasm_bindgen(start)]
pub fn main() {
    console_log!("WASM Game Engine v1.8 Classic Edition loaded successfully!");
}