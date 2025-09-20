use wasm_bindgen::prelude::*;
use js_sys::*;
use web_sys::*;
use std::collections::HashMap;
use nalgebra::{Vector3, Matrix4, UnitQuaternion};
use serde::{Serialize, Deserialize};

// Web-Optimized Game Engine for Deplauncher 1.12 - Enhanced Edition (Rust)
// Specifically optimized for web browsers with modern WebGL and Canvas API

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
    
    #[wasm_bindgen(js_namespace = performance)]
    fn now() -> f64;
    
    #[wasm_bindgen(js_namespace = navigator)]
    fn hardwareConcurrency() -> u32;
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

// === WEB COLLISION SYSTEM ===

#[derive(Debug)]
pub struct WebCollisionSystem {
    spatial_grid: HashMap<(i32, i32), Vec<u32>>,
    cell_size: f32,
}

impl WebCollisionSystem {
    pub fn new() -> Self {
        Self {
            spatial_grid: HashMap::new(),
            cell_size: 64.0,
        }
    }
    
    pub fn update(&mut self, entities: &mut HashMap<u32, WebEntity>) -> i32 {
        // Clear spatial grid
        self.spatial_grid.clear();
        
        // Populate spatial grid
        for (&id, entity) in entities.iter() {
            if !entity.active {
                continue;
            }
            
            let grid_x = (entity.transform.position.x / self.cell_size) as i32;
            let grid_y = (entity.transform.position.y / self.cell_size) as i32;
            
            self.spatial_grid.entry((grid_x, grid_y))
                .or_insert_with(Vec::new)
                .push(id);
        }
        
        let mut score_increment = 0;
        let mut collisions = Vec::new();
        
        // Check collisions within grid cells
        for entity_ids in self.spatial_grid.values() {
            for i in 0..entity_ids.len() {
                for j in (i + 1)..entity_ids.len() {
                    let id_a = entity_ids[i];
                    let id_b = entity_ids[j];
                    
                    if let (Some(entity_a), Some(entity_b)) = (entities.get(&id_a), entities.get(&id_b)) {
                        let distance = (entity_a.transform.position - entity_b.transform.position).magnitude();
                        let collision_radius = entity_a.physics.as_ref().map(|p| p.collision_radius).unwrap_or(16.0) +
                                             entity_b.physics.as_ref().map(|p| p.collision_radius).unwrap_or(16.0);
                        
                        if distance < collision_radius {
                            collisions.push((id_a, id_b, distance, collision_radius));
                            
                            // Score for player collisions
                            if entity_a.tag == "Player" || entity_b.tag == "Player" {
                                score_increment += 10;
                            }
                        }
                    }
                }
            }
        }
        
        // Resolve collisions
        for (id_a, id_b, distance, collision_radius) in collisions {
            if let (Some(entity_a), Some(entity_b)) = (entities.get_mut(&id_a), entities.get_mut(&id_b)) {
                let direction = (entity_a.transform.position - entity_b.transform.position).normalize();
                let overlap = collision_radius - distance;
                
                // Separate entities
                entity_a.transform.position += direction * overlap * 0.5;
                entity_b.transform.position -= direction * overlap * 0.5;
                
                // Apply collision response
                if let (Some(physics_a), Some(physics_b)) = (&entity_a.physics, &entity_b.physics) {
                    let bounce_force = 100.0 * (physics_a.bounciness + physics_b.bounciness) * 0.5;
                    
                    entity_a.transform.velocity += direction * bounce_force;
                    entity_b.transform.velocity -= direction * bounce_force;
                }
            }
        }
        
        score_increment
    }
}

// === MAIN WEB GAME STATE ===

#[wasm_bindgen]
pub struct WebGameState {
    // Entity management
    entities: HashMap<u32, WebEntity>,
    next_entity_id: u32,
    
    // Systems
    particle_system: WebParticleSystem,
    collision_system: WebCollisionSystem,
    performance: WebPerformanceMonitor,
    input: WebInputSystem,
    
    // Browser capabilities
    capabilities: BrowserCapabilities,
    
    // Camera
    camera_position: Vector3<f32>,
    camera_target: Vector3<f32>,
    camera_fov: f32,
    
    // Physics
    gravity: Vector3<f32>,
    physics_enabled: bool,
    
    // Game state
    score: i32,
    level: i32,
    time_scale: f32,
    paused: bool,
    debug_mode: bool,
}

#[wasm_bindgen]
impl WebGameState {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_log!("Creating Web Game State for v1.12 Enhanced Edition");
        
        let capabilities = BrowserCapabilities::default();
        
        let mut game_state = Self {
            entities: HashMap::with_capacity(MAX_ENTITIES),
            next_entity_id: 1,
            
            particle_system: WebParticleSystem::new(),
            collision_system: WebCollisionSystem::new(),
            performance: WebPerformanceMonitor::new(),
            input: WebInputSystem::new(),
            
            capabilities,
            
            camera_position: Vector3::new(CANVAS_WIDTH / 2.0, CANVAS_HEIGHT / 2.0, -500.0),
            camera_target: Vector3::new(CANVAS_WIDTH / 2.0, CANVAS_HEIGHT / 2.0, 0.0),
            camera_fov: 75.0,
            
            gravity: Vector3::new(0.0, -490.0, 0.0), // Reduced for web
            physics_enabled: true,
            
            score: 0,
            level: 1,
            time_scale: 1.0,
            paused: false,
            debug_mode: false,
        };
        
        game_state.initialize_scene();
        game_state
    }
    
    fn initialize_scene(&mut self) {
        // Create player entity
        let player_id = self.create_entity("Player".to_string(), Vector3::new(CANVAS_WIDTH / 2.0, CANVAS_HEIGHT / 2.0, 0.0));
        
        if let Some(player) = self.entities.get_mut(&player_id) {
            player.add_physics(WebPhysics {
                mass: 1.0,
                use_gravity: false, // Top-down view
                drag: 5.0,
                ..Default::default()
            });
            
            player.add_renderer(WebRenderer {
                color: [0.3, 0.8, 1.0, 1.0], // Cyan
                ..Default::default()
            });
            
            player.add_health(100.0);
            player.tag = "Player".to_string();
        }
        
        // Generate environment entities (reduced for web)
        self.generate_environment(50);
        
        console_log!("Scene initialized with {} entities", self.entities.len());
    }
    
    fn generate_environment(&mut self, count: usize) {
        for i in 0..count {
            let position = Vector3::new(
                Math::random() as f32 * CANVAS_WIDTH,
                Math::random() as f32 * CANVAS_HEIGHT,
                0.0,
            );
            
            let entity_id = self.create_entity(format!("Environment_{}", i), position);
            
            if let Some(entity) = self.entities.get_mut(&entity_id) {
                entity.add_physics(WebPhysics {
                    mass: 0.5 + Math::random() as f32 * 2.0,
                    bounciness: 0.3 + Math::random() as f32 * 0.7,
                    drag: 0.1 + Math::random() as f32 * 0.8,
                    ..Default::default()
                });
                
                entity.add_renderer(WebRenderer {
                    color: [
                        0.5 + Math::random() as f32 * 0.5,
                        0.5 + Math::random() as f32 * 0.5,
                        0.5 + Math::random() as f32 * 0.5,
                        1.0,
                    ],
                    ..Default::default()
                });
                
                entity.tag = "Environment".to_string();
            }
        }
    }
    
    pub fn create_entity(&mut self, name: String, position: Vector3<f32>) -> u32 {
        let id = self.next_entity_id;
        self.next_entity_id += 1;
        
        let entity = WebEntity::new(id, name, position);
        self.entities.insert(id, entity);
        
        id
    }
    
    #[wasm_bindgen]
    pub fn update(&mut self, current_time: f64) {
        if self.paused {
            return;
        }
        
        let delta_time = self.performance.update(current_time) * self.time_scale;
        
        // Update input
        let movement = self.input.get_movement_input();
        if movement.magnitude() > 0.1 {
            if let Some(player) = self.entities.get_mut(&1) {
                let move_speed = 300.0;
                player.transform.acceleration += movement * move_speed;
            }
        }
        
        // Update entities based on quality level
        for entity in self.entities.values_mut() {
            entity.update(delta_time);
        }
        
        // Update systems based on performance level
        if self.performance.quality_level >= 1 {
            if self.physics_enabled {
                self.update_physics_system(delta_time);
            }
            
            let score_increment = self.collision_system.update(&mut self.entities);
            self.score += score_increment;
        }
        
        if self.performance.quality_level >= 2 {
            self.particle_system.update(delta_time);
        }
        
        // Update camera to follow player
        if let Some(player) = self.entities.get(&1) {
            let lerp_factor = 3.0 * delta_time;
            let target = player.transform.position;
            self.camera_target = self.camera_target.lerp(&target, lerp_factor);
        }
        
        // Cleanup dead entities
        self.entities.retain(|_, entity| entity.is_alive());
        
        // Debug output
        if self.debug_mode && self.performance.fps_counter % 60 == 0 {
            console_log!("FPS: {:.1}, Entities: {}, Particles: {}, Quality: {}", 
                        self.performance.current_fps, 
                        self.entities.len(), 
                        self.particle_system.particle_count(),
                        self.performance.quality_level);
        }
    }
    
    fn update_physics_system(&mut self, delta_time: f32) {
        for entity in self.entities.values_mut() {
            if !entity.active {
                continue;
            }
            
            if let Some(physics) = &entity.physics {
                if !physics.is_kinematic && physics.use_gravity {
                    entity.transform.acceleration += self.gravity;
                }
            }
        }
    }
    
    // === WASM EXPORTS ===
    
    #[wasm_bindgen]
    pub fn handle_key_event(&mut self, key_code: u32, pressed: bool) {
        self.input.set_key(key_code, pressed);
        
        if pressed {
            match key_code {
                32 => { // Space
                    self.paused = !self.paused;
                    console_log!("Game {}", if self.paused { "paused" } else { "resumed" });
                }
                192 => { // Tilde (~)
                    self.debug_mode = !self.debug_mode;
                    console_log!("Debug mode {}", if self.debug_mode { "enabled" } else { "disabled" });
                }
                _ => {}
            }
        }
    }
    
    #[wasm_bindgen]
    pub fn handle_mouse_event(&mut self, x: f32, y: f32, delta_x: f32, delta_y: f32) {
        self.input.set_mouse(x, y, delta_x, delta_y);
    }
    
    #[wasm_bindgen]
    pub fn handle_touch_event(&mut self, touches: Vec<f32>) {
        let touch_pairs: Vec<(f32, f32)> = touches
            .chunks_exact(2)
            .map(|chunk| (chunk[0], chunk[1]))
            .collect();
        self.input.set_touch(touch_pairs);
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
    pub fn get_fps(&self) -> f32 {
        self.performance.current_fps
    }
    
    #[wasm_bindgen]
    pub fn get_frame_time(&self) -> f32 {
        self.performance.average_frame_time_ms
    }
    
    #[wasm_bindgen]
    pub fn get_quality_level(&self) -> u8 {
        self.performance.quality_level
    }
    
    #[wasm_bindgen]
    pub fn set_quality_level(&mut self, quality: u8) {
        self.performance.quality_level = quality.min(2);
        self.performance.adaptive_quality = false;
        console_log!("Quality manually set to {}", self.performance.quality_level);
    }
    
    #[wasm_bindgen]
    pub fn enable_adaptive_quality(&mut self, enabled: bool) {
        self.performance.adaptive_quality = enabled;
        console_log!("Adaptive quality {}", if enabled { "enabled" } else { "disabled" });
    }
    
    #[wasm_bindgen]
    pub fn get_entity_render_data(&self) -> Vec<f32> {
        let mut data = Vec::with_capacity(self.entities.len() * 16);
        
        for entity in self.entities.values() {
            if !entity.active || !entity.renderer.as_ref().map_or(false, |r| r.visible) {
                continue;
            }
            
            let transform = &entity.transform;
            let renderer = entity.renderer.as_ref().unwrap();
            
            // Transform matrix (simplified 2D)
            data.extend_from_slice(&[
                transform.scale, 0.0, 0.0, transform.position.x,
                0.0, transform.scale, 0.0, transform.position.y,
                0.0, 0.0, transform.scale, transform.position.z,
                renderer.color[0], renderer.color[1], renderer.color[2], 
                renderer.color[3] * renderer.opacity,
            ]);
        }
        
        data
    }
    
    #[wasm_bindgen]
    pub fn get_particle_render_data(&self) -> Vec<f32> {
        self.particle_system.get_render_data()
    }
    
    #[wasm_bindgen]
    pub fn get_camera_data(&self) -> Vec<f32> {
        vec![
            self.camera_position.x, self.camera_position.y, self.camera_position.z,
            self.camera_target.x, self.camera_target.y, self.camera_target.z,
            self.camera_fov,
        ]
    }
    
    #[wasm_bindgen]
    pub fn create_explosion(&mut self, x: f32, y: f32, z: f32, intensity: f32) {
        let position = Vector3::new(x, y, z);
        self.particle_system.create_explosion(position, intensity);
    }
    
    #[wasm_bindgen]
    pub fn add_entity(&mut self, x: f32, y: f32, z: f32, name: String, tag: String) -> u32 {
        let position = Vector3::new(x, y, z);
        let entity_id = self.create_entity(name, position);
        
        if let Some(entity) = self.entities.get_mut(&entity_id) {
            entity.tag = tag;
            
            entity.add_physics(WebPhysics::default());
            entity.add_renderer(WebRenderer::default());
        }
        
        entity_id
    }
    
    #[wasm_bindgen]
    pub fn set_browser_capabilities(&mut self, 
                                   webgl2: bool, 
                                   hardware_accel: bool, 
                                   is_mobile: bool, 
                                   cpu_cores: u32) {
        self.capabilities.webgl2_available = webgl2;
        self.capabilities.hardware_acceleration = hardware_accel;
        self.capabilities.is_mobile = is_mobile;
        self.capabilities.cpu_cores = cpu_cores;
        
        // Adjust performance based on capabilities
        if is_mobile || !hardware_accel {
            self.performance.quality_level = 1; // Start with medium quality on mobile/slow devices
        }
        
        console_log!("Browser capabilities updated: WebGL2={}, HW Accel={}, Mobile={}, Cores={}", 
                    webgl2, hardware_accel, is_mobile, cpu_cores);
    }
    
    #[wasm_bindgen]
    pub fn get_performance_info(&self) -> JsValue {
        let info = js_sys::Object::new();
        
        js_sys::Reflect::set(&info, &"fps".into(), &self.performance.current_fps.into()).unwrap();
        js_sys::Reflect::set(&info, &"frameTime".into(), &self.performance.average_frame_time_ms.into()).unwrap();
        js_sys::Reflect::set(&info, &"qualityLevel".into(), &self.performance.quality_level.into()).unwrap();
        js_sys::Reflect::set(&info, &"adaptiveQuality".into(), &self.performance.adaptive_quality.into()).unwrap();
        js_sys::Reflect::set(&info, &"entityCount".into(), &self.entities.len().into()).unwrap();
        js_sys::Reflect::set(&info, &"particleCount".into(), &self.particle_system.particle_count().into()).unwrap();
        js_sys::Reflect::set(&info, &"droppedFrames".into(), &self.performance.dropped_frames.into()).unwrap();
        
        info.into()
    }
    
    #[wasm_bindgen]
    pub fn reset_game(&mut self) {
        console_log!("Resetting web game state");
        
        self.entities.clear();
        self.next_entity_id = 1;
        self.score = 0;
        self.level = 1;
        self.paused = false;
        
        self.particle_system = WebParticleSystem::new();
        self.collision_system = WebCollisionSystem::new();
        
        self.initialize_scene();
    }
    
    #[wasm_bindgen]
    pub fn cleanup(&mut self) {
        console_log!("Cleaning up Web Game Engine v1.12");
        self.entities.clear();
        self.particle_system = WebParticleSystem::new();
    }
}

// === WASM ENGINE WRAPPER ===

#[wasm_bindgen]
pub struct WebGameEngine {
    game_state: WebGameState,
}

#[wasm_bindgen]
impl WebGameEngine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WebGameEngine {
        console_log!("Initializing WASM Web Game Engine v1.12");
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        
        WebGameEngine {
            game_state: WebGameState::new(),
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
    pub fn handle_mouse(&mut self, x: f32, y: f32, delta_x: f32, delta_y: f32) {
        self.game_state.handle_mouse_event(x, y, delta_x, delta_y);
    }
    
    #[wasm_bindgen]
    pub fn handle_touch(&mut self, touches: Vec<f32>) {
        self.game_state.handle_touch_event(touches);
    }
    
    #[wasm_bindgen]
    pub fn get_render_data(&self) -> JsValue {
        let data = js_sys::Object::new();
        
        let entities = self.game_state.get_entity_render_data();
        js_sys::Reflect::set(&data, &"entities".into(), 
                           &js_sys::Float32Array::from(&entities[..]).into()).unwrap();
        
        let particles = self.game_state.get_particle_render_data();
        js_sys::Reflect::set(&data, &"particles".into(), 
                           &js_sys::Float32Array::from(&particles[..]).into()).unwrap();
        
        let camera = self.game_state.get_camera_data();
        js_sys::Reflect::set(&data, &"camera".into(), 
                           &js_sys::Float32Array::from(&camera[..]).into()).unwrap();
        
        data.into()
    }
    
    #[wasm_bindgen]
    pub fn get_performance_info(&self) -> JsValue {
        self.game_state.get_performance_info()
    }
    
    #[wasm_bindgen]
    pub fn set_browser_capabilities(&mut self, webgl2: bool, hardware_accel: bool, is_mobile: bool, cpu_cores: u32) {
        self.game_state.set_browser_capabilities(webgl2, hardware_accel, is_mobile, cpu_cores);
    }
    
    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.game_state.reset_game();
    }
    
    #[wasm_bindgen]
    pub fn cleanup(&mut self) {
        self.game_state.cleanup();
    }
}

// === ENTRY POINT ===

#[wasm_bindgen(start)]
pub fn main() {
    console_log!("WASM Web Game Engine v1.12 Enhanced Edition loaded successfully!");
    console_log!("Features: WebGL Optimized, Adaptive Quality, Mobile Support, Performance Monitoring");
} WEB-SPECIFIC CONSTANTS ===
const MAX_ENTITIES: usize = 2000;        // Optimized for web
const MAX_PARTICLES: usize = 5000;       // WebGL friendly
const MAX_LIGHTS: usize = 25;            // WebGL shader limit
const CANVAS_WIDTH: f32 = 1920.0;
const CANVAS_HEIGHT: f32 = 1080.0;
const TARGET_FPS: f32 = 60.0;
const MAX_FRAME_TIME_MS: f32 = 16.67;    // 60 FPS budget
const QUALITY_ADJUSTMENT_COOLDOWN: u32 = 60; // Frames

// === WEB BROWSER DETECTION ===

#[derive(Debug, Clone)]
pub struct BrowserCapabilities {
    pub webgl2_available: bool,
    pub hardware_acceleration: bool,
    pub max_texture_size: u32,
    pub max_vertex_attribs: u32,
    pub instanced_rendering: bool,
    pub vertex_array_objects: bool,
    pub is_mobile: bool,
    pub cpu_cores: u32,
    pub estimated_performance_tier: u8, // 0=low, 1=medium, 2=high
}

impl Default for BrowserCapabilities {
    fn default() -> Self {
        Self {
            webgl2_available: true,
            hardware_acceleration: true,
            max_texture_size: 4096,
            max_vertex_attribs: 16,
            instanced_rendering: true,
            vertex_array_objects: true,
            is_mobile: false,
            cpu_cores: 4,
            estimated_performance_tier: 2,
        }
    }
}

// === WEB-OPTIMIZED COMPONENTS ===

#[derive(Debug, Clone, Copy)]
pub struct WebTransform {
    pub position: Vector3<f32>,
    pub velocity: Vector3<f32>,
    pub acceleration: Vector3<f32>,
    pub rotation: f32,        // Single Y-axis rotation for web performance
    pub scale: f32,          // Uniform scale
}

impl Default for WebTransform {
    fn default() -> Self {
        Self {
            position: Vector3::zeros(),
            velocity: Vector3::zeros(),
            acceleration: Vector3::zeros(),
            rotation: 0.0,
            scale: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WebPhysics {
    pub mass: f32,
    pub drag: f32,
    pub bounciness: f32,
    pub is_kinematic: bool,
    pub use_gravity: bool,
    pub collision_radius: f32,
}

impl Default for WebPhysics {
    fn default() -> Self {
        Self {
            mass: 1.0,
            drag: 0.02,
            bounciness: 0.5,
            is_kinematic: false,
            use_gravity: true,
            collision_radius: 16.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WebRenderer {
    pub texture_id: u32,
    pub color: [f32; 4],
    pub opacity: f32,
    pub visible: bool,
    pub render_layer: u8,
    pub blend_mode: BlendMode,
}

#[derive(Debug, Clone)]
pub enum BlendMode {
    Normal,
    Additive,
    Multiply,
    Screen,
}

impl Default for WebRenderer {
    fn default() -> Self {
        Self {
            texture_id: 0,
            color: [1.0, 1.0, 1.0, 1.0],
            opacity: 1.0,
            visible: true,
            render_layer: 0,
            blend_mode: BlendMode::Normal,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WebHealth {
    pub current: f32,
    pub max: f32,
    pub regeneration: f32,
    pub last_damage_time: f32,
}

impl Default for WebHealth {
    fn default() -> Self {
        Self {
            current: 100.0,
            max: 100.0,
            regeneration: 0.0,
            last_damage_time: 0.0,
        }
    }
}

// === WEB ENTITY SYSTEM ===

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct WebEntity {
    pub id: u32,
    pub name: String,
    pub tag: String,
    pub active: bool,
    
    // Core components
    pub transform: WebTransform,
    pub physics: Option<WebPhysics>,
    pub renderer: Option<WebRenderer>,
    pub health: Option<WebHealth>,
}

impl WebEntity {
    pub fn new(id: u32, name: String, position: Vector3<f32>) -> Self {
        Self {
            id,
            name,
            tag: "Default".to_string(),
            active: true,
            transform: WebTransform {
                position,
                ..Default::default()
            },
            physics: None,
            renderer: None,
            health: None,
        }
    }
    
    pub fn add_physics(&mut self, physics: WebPhysics) -> &mut Self {
        self.physics = Some(physics);
        self
    }
    
    pub fn add_renderer(&mut self, renderer: WebRenderer) -> &mut Self {
        self.renderer = Some(renderer);
        self
    }
    
    pub fn add_health(&mut self, max_health: f32) -> &mut Self {
        self.health = Some(WebHealth {
            current: max_health,
            max: max_health,
            ..Default::default()
        });
        self
    }
    
    pub fn is_alive(&self) -> bool {
        self.active && self.health.as_ref().map_or(true, |h| h.current > 0.0)
    }
    
    pub fn update(&mut self, delta_time: f32) {
        if !self.active {
            return;
        }
        
        // Update physics
        if let Some(physics) = &self.physics {
            if !physics.is_kinematic {
                // Apply drag
                self.transform.velocity *= 1.0 - (physics.drag * delta_time);
                
                // Integration
                self.transform.velocity += self.transform.acceleration * delta_time;
                self.transform.position += self.transform.velocity * delta_time;
                
                // Reset acceleration
                self.transform.acceleration = Vector3::zeros();
            }
        }
        
        // Update rotation (simple spinning for visual effect)
        self.transform.rotation += delta_time * 45.0; // 45 degrees per second
        if self.transform.rotation > 360.0 {
            self.transform.rotation -= 360.0;
        }
        
        // Boundary wrapping for web canvas
        if self.transform.position.x < 0.0 {
            self.transform.position.x = CANVAS_WIDTH;
        } else if self.transform.position.x > CANVAS_WIDTH {
            self.transform.position.x = 0.0;
        }
        
        if self.transform.position.y < 0.0 {
            self.transform.position.y = CANVAS_HEIGHT;
        } else if self.transform.position.y > CANVAS_HEIGHT {
            self.transform.position.y = 0.0;
        }
        
        // Update health regeneration
        if let Some(health) = &mut self.health {
            if health.regeneration != 0.0 {
                health.current += health.regeneration * delta_time;
                health.current = health.current.min(health.max).max(0.0);
            }
        }
    }
}

// === WEB PARTICLE SYSTEM ===

#[derive(Debug, Clone)]
pub struct WebParticle {
    pub position: Vector3<f32>,
    pub velocity: Vector3<f32>,
    pub color: [f32; 4],
    pub life: f32,
    pub max_life: f32,
    pub size: f32,
    pub rotation: f32,
    pub angular_velocity: f32,
    pub active: bool,
}

#[derive(Debug)]
pub struct WebParticleSystem {
    particles: Vec<WebParticle>,
    gravity: Vector3<f32>,
    next_cleanup: u32,
}

impl WebParticleSystem {
    pub fn new() -> Self {
        Self {
            particles: Vec::with_capacity(MAX_PARTICLES),
            gravity: Vector3::new(0.0, -98.0, 0.0),
            next_cleanup: 0,
        }
    }
    
    pub fn update(&mut self, delta_time: f32) {
        // Update existing particles
        for particle in &mut self.particles {
            if !particle.active {
                continue;
            }
            
            // Apply forces
            particle.velocity += self.gravity * delta_time;
            particle.position += particle.velocity * delta_time;
            particle.rotation += particle.angular_velocity * delta_time;
            
            // Update life
            particle.life -= delta_time;
            if particle.life <= 0.0 {
                particle.active = false;
                continue;
            }
            
            // Update visual properties
            let life_ratio = particle.life / particle.max_life;
            particle.color[3] = life_ratio; // Alpha fade
            particle.size *= 0.995; // Gradual shrink
        }
        
        // Periodic cleanup (every 60 frames at 60fps = 1 second)
        self.next_cleanup = self.next_cleanup.wrapping_add(1);
        if self.next_cleanup % 60 == 0 {
            self.particles.retain(|p| p.active);
        }
    }
    
    pub fn create_explosion(&mut self, position: Vector3<f32>, intensity: f32) {
        let particle_count = (intensity * 30.0) as usize;
        let max_new_particles = (MAX_PARTICLES - self.particles.len()).min(particle_count).min(50);
        
        for _ in 0..max_new_particles {
            let angle = Math::random() * 2.0 * std::f64::consts::PI;
            let elevation = Math::random() * std::f64::consts::PI - std::f64::consts::PI * 0.5;
            let speed = 80.0 + Math::random() * 120.0;
            
            let velocity = Vector3::new(
                (angle.cos() * elevation.cos() * speed) as f32,
                (elevation.sin() * speed) as f32,
                (angle.sin() * elevation.cos() * speed) as f32,
            );
            
            let particle = WebParticle {
                position,
                velocity,
                color: [1.0, 0.7, 0.2, 1.0], // Orange fire
                life: 1.0 + Math::random() as f32,
                max_life: 2.0,
                size: 3.0 + Math::random() as f32 * 4.0,
                rotation: 0.0,
                angular_velocity: (Math::random() as f32 - 0.5) * 10.0,
                active: true,
            };
            
            self.particles.push(particle);
        }
    }
    
    pub fn particle_count(&self) -> usize {
        self.particles.iter().filter(|p| p.active).count()
    }
    
    pub fn get_render_data(&self) -> Vec<f32> {
        let active_particles: Vec<&WebParticle> = self.particles.iter().filter(|p| p.active).collect();
        let mut data = Vec::with_capacity(active_particles.len() * 8);
        
        for particle in active_particles {
            data.extend_from_slice(&[
                particle.position.x,
                particle.position.y,
                particle.position.z,
                particle.size,
                particle.color[0],
                particle.color[1],
                particle.color[2],
                particle.color[3],
            ]);
        }
        
        data
    }
}

// === WEB PERFORMANCE SYSTEM ===

#[derive(Debug)]
pub struct WebPerformanceMonitor {
    pub last_frame_time: f64,
    pub frame_times: Vec<f32>,
    pub fps_counter: u32,
    pub fps_timer: f64,
    pub current_fps: f32,
    pub average_frame_time_ms: f32,
    pub quality_level: u8,
    pub adaptive_quality: bool,
    pub quality_cooldown: u32,
    pub dropped_frames: u32,
}

impl WebPerformanceMonitor {
    pub fn new() -> Self {
        Self {
            last_frame_time: now(),
            frame_times: Vec::with_capacity(60),
            fps_counter: 0,
            fps_timer: 0.0,
            current_fps: 60.0,
            average_frame_time_ms: 16.67,
            quality_level: 2, // Start with high quality
            adaptive_quality: true,
            quality_cooldown: 0,
            dropped_frames: 0,
        }
    }
    
    pub fn update(&mut self, current_time: f64) -> f32 {
        let frame_time = ((current_time - self.last_frame_time) / 1000.0) as f32;
        self.last_frame_time = current_time;
        
        // Cap delta time
        let capped_frame_time = frame_time.min(0.033); // Max 33ms
        
        // Track frame time
        self.frame_times.push(frame_time * 1000.0); // Convert to ms
        if self.frame_times.len() > 60 {
            self.frame_times.remove(0);
        }
        
        // Calculate average frame time
        if !self.frame_times.is_empty() {
            self.average_frame_time_ms = self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32;
        }
        
        // Update FPS
        self.fps_counter += 1;
        self.fps_timer += frame_time as f64;
        if self.fps_timer >= 1.0 {
            self.current_fps = self.fps_counter as f32 / self.fps_timer as f32;
            self.fps_counter = 0;
            self.fps_timer = 0.0;
        }
        
        // Adaptive quality adjustment
        if self.adaptive_quality && self.quality_cooldown == 0 {
            if self.average_frame_time_ms > MAX_FRAME_TIME_MS * 1.3 {
                if self.quality_level > 0 {
                    self.quality_level -= 1;
                    self.quality_cooldown = QUALITY_ADJUSTMENT_COOLDOWN;
                    console_log!("Quality reduced to {} (frame time: {:.2}ms)", 
                                self.quality_level, self.average_frame_time_ms);
                }
            } else if self.average_frame_time_ms < MAX_FRAME_TIME_MS * 0.7 {
                if self.quality_level < 2 {
                    self.quality_level += 1;
                    self.quality_cooldown = QUALITY_ADJUSTMENT_COOLDOWN * 3; // Longer cooldown for increase
                    console_log!("Quality increased to {} (frame time: {:.2}ms)", 
                                self.quality_level, self.average_frame_time_ms);
                }
            }
        }
        
        if self.quality_cooldown > 0 {
            self.quality_cooldown -= 1;
        }
        
        capped_frame_time
    }
    
    pub fn is_performance_good(&self) -> bool {
        self.current_fps >= TARGET_FPS * 0.9 && self.average_frame_time_ms <= MAX_FRAME_TIME_MS * 1.1
    }
}

// === WEB INPUT SYSTEM ===

#[derive(Debug, Default)]
pub struct WebInputSystem {
    pub keys: HashMap<u32, bool>,
    pub mouse_pos: Vector3<f32>,
    pub mouse_delta: Vector3<f32>,
    pub mouse_buttons: [bool; 3],
    pub touches: Vec<(f32, f32)>,
    pub touch_active: bool,
}

impl WebInputSystem {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn set_key(&mut self, key_code: u32, pressed: bool) {
        self.keys.insert(key_code, pressed);
    }
    
    pub fn is_key_pressed(&self, key_code: u32) -> bool {
        *self.keys.get(&key_code).unwrap_or(&false)
    }
    
    pub fn set_mouse(&mut self, x: f32, y: f32, delta_x: f32, delta_y: f32) {
        self.mouse_pos = Vector3::new(x, y, 0.0);
        self.mouse_delta = Vector3::new(delta_x, delta_y, 0.0);
    }
    
    pub fn set_touch(&mut self, touches: Vec<(f32, f32)>) {
        self.touches = touches;
        self.touch_active = !self.touches.is_empty();
    }
    
    pub fn get_movement_input(&self) -> Vector3<f32> {
        let mut movement = Vector3::zeros();
        
        // WASD movement
        if self.is_key_pressed(87) { movement.y -= 1.0; } // W
        if self.is_key_pressed(83) { movement.y += 1.0; } // S
        if self.is_key_pressed(65) { movement.x -= 1.0; } // A
        if self.is_key_pressed(68) { movement.x += 1.0; } // D
        
        // Arrow keys
        if self.is_key_pressed(38) { movement.y -= 1.0; } // Up
        if self.is_key_pressed(40) { movement.y += 1.0; } // Down
        if self.is_key_pressed(37) { movement.x -= 1.0; } // Left
        if self.is_key_pressed(39) { movement.x += 1.0; } // Right
        
        // Normalize diagonal movement
        if movement.magnitude() > 1.0 {
            movement = movement.normalize();
        }
        
        movement
    }
}
// ===
