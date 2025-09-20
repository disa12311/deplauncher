use wasm_bindgen::prelude::*;
use js_sys::*;
use web_sys::*;
use std::collections::HashMap;

// Web-Optimized Game Engine for Deplauncher 1.8 - Classic Edition (Rust)
// Lightweight, memory-safe engine specifically optimized for web browsers

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
    
    #[wasm_bindgen(js_namespace = performance)]
    fn now() -> f64;
    
    #[wasm_bindgen(js_namespace = navigator)]
    fn hardwareConcurrency() -> u32;
    
    #[wasm_bindgen(js_namespace = navigator)]
    fn userAgent() -> String;
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

// === WEB-SPECIFIC CONSTANTS ===
const MAX_ENTITIES: usize = 800;         // Optimized for web browsers
const MAX_PARTICLES: usize = 200;        // Reduced for consistent performance
const CANVAS_WIDTH: f32 = 800.0;
const CANVAS_HEIGHT: f32 = 600.0;
const COLLISION_RADIUS: f32 = 28.0;      // Slightly reduced for responsiveness
const TARGET_FPS: f32 = 60.0;
const MAX_FRAME_TIME_MS: f32 = 16.67;    // 60 FPS budget

// === WEB BROWSER DETECTION ===

#[derive(Debug, Clone)]
pub struct BrowserInfo {
    pub is_mobile: bool,
    pub is_low_end: bool,
    pub supports_webgl2: bool,
    pub cpu_cores: u32,
    pub performance_tier: u8, // 0=low, 1=medium, 2=high
}

impl Default for BrowserInfo {
    fn default() -> Self {
        Self {
            is_mobile: false,
            is_low_end: false,
            supports_webgl2: true,
            cpu_cores: 4,
            performance_tier: 2,
        }
    }
}

impl BrowserInfo {
    pub fn detect() -> Self {
        let mut info = Self::default();
        
        // Detect mobile
        let user_agent = userAgent();
        info.is_mobile = user_agent.contains("Mobile") || 
                        user_agent.contains("Android") || 
                        user_agent.contains("iPhone");
        
        // Get CPU cores
        info.cpu_cores = hardwareConcurrency().max(1);
        
        // Estimate performance tier
        if info.is_mobile || info.cpu_cores <= 2 {
            info.performance_tier = 0; // Low
            info.is_low_end = true;
        } else if info.cpu_cores <= 4 {
            info.performance_tier = 1; // Medium
        } else {
            info.performance_tier = 2; // High
        }
        
        info
    }
}

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
    
    pub fn magnitude_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }
    
    pub fn magnitude(&self) -> f32 {
        self.magnitude_squared().sqrt()
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
    
    pub fn distance_squared_to(&self, other: &Vector2) -> f32 {
        (*self - *other).magnitude_squared()
    }
    
    pub fn lerp(&self, target: &Vector2, t: f32) -> Self {
        *self + (*target - *self) * t
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

// === WEB ENTITY SYSTEM ===

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct WebEntity {
    pub position: Vector2,
    pub velocity: Vector2,
    pub rotation: f32,
    pub texture_id: u32,
    pub active: bool,
    pub health: i32,
    pub max_health: i32,
    pub name: String,
    pub entity_type: EntityType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EntityType {
    Player,
    Environment,
    Pickup,
    Projectile,
}

impl WebEntity {
    pub fn new(position: Vector2, texture_id: u32, name: String, entity_type: EntityType) -> Self {
        Self {
            position,
            velocity: Vector2::zero(),
            rotation: 0.0,
            texture_id,
            active: true,
            health: 100,
            max_health: 100,
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
        self.rotation += delta_time * 60.0; // 60 degrees per second
        if self.rotation > 360.0 {
            self.rotation -= 360.0;
        }
        
        // Boundary wrapping for classic arcade feel
        if self.position.x < -32.0 {
            self.position.x = CANVAS_WIDTH + 32.0;
        } else if self.position.x > CANVAS_WIDTH + 32.0 {
            self.position.x = -32.0;
        }
        
        if self.position.y < -32.0 {
            self.position.y = CANVAS_HEIGHT + 32.0;
        } else if self.position.y > CANVAS_HEIGHT + 32.0 {
            self.position.y = -32.0;
        }
        
        // Apply friction for classic physics
        self.velocity *= 0.92; // More responsive than original
    }
    
    pub fn apply_force(&mut self, force: Vector2) {
        self.velocity += force;
    }
    
    pub fn is_alive(&self) -> bool {
        self.active && self.health > 0
    }
    
    pub fn take_damage(&mut self, damage: i32) {
        self.health = (self.health - damage).max(0);
        if self.health <= 0 {
            self.active = false;
        }
    }
}

// === WEB PARTICLE SYSTEM ===

#[derive(Debug, Clone)]
pub struct WebParticle {
    pub position: Vector2,
    pub velocity: Vector2,
    pub life: f32,
    pub max_life: f32,
    pub size: f32,
    pub color: [u8; 3],
    pub active: bool,
}

impl WebParticle {
    pub fn new(position: Vector2, velocity: Vector2, life: f32, size: f32, color: [u8; 3]) -> Self {
        Self {
            position,
            velocity,
            life,
            max_life: life,
            size,
            color,
            active: true,
        }
    }
    
    pub fn update(&mut self, delta_time: f32) {
        if !self.active {
            return;
        }
        
        // Apply gravity
        self.velocity.y += 120.0 * delta_time;
        
        // Update position
        self.position += self.velocity * delta_time;
        
        // Update life
        self.life -= delta_time;
        if self.life <= 0.0 {
            self.active = false;
            return;
        }
        
        // Update size
        self.size *= 0.98; // Gradual shrinking
    }
    
    pub fn life_ratio(&self) -> f32 {
        if self.max_life > 0.0 {
            (self.life / self.max_life).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }
    
    pub fn alpha(&self) -> f32 {
        self.life_ratio()
    }
}

#[derive(Debug)]
pub struct WebParticleSystem {
    particles: Vec<WebParticle>,
    max_particles: usize,
    cleanup_timer: u32,
}

impl WebParticleSystem {
    pub fn new(max_particles: usize) -> Self {
        Self {
            particles: Vec::with_capacity(max_particles),
            max_particles,
            cleanup_timer: 0,
        }
    }
    
    pub fn update(&mut self, delta_time: f32) {
        // Update existing particles
        for particle in &mut self.particles {
            particle.update(delta_time);
        }
        
        // Periodic cleanup to maintain performance
        self.cleanup_timer += 1;
        if self.cleanup_timer >= 120 { // Every 2 seconds at 60fps
            self.particles.retain(|p| p.active);
            self.cleanup_timer = 0;
        }
    }
    
    pub fn create_explosion(&mut self, position: Vector2, count: usize) {
        let actual_count = count.min(15).min(self.max_particles - self.particles.len());
        
        for _ in 0..actual_count {
            let angle = Math::random() * 2.0 * std::f64::consts::PI;
            let speed = 60.0 + Math::random() * 80.0;
            
            let velocity = Vector2::new(
                (angle.cos() * speed) as f32,
                (angle.sin() * speed) as f32,
            );
            
            let life = 1.0 + Math::random() as f32 * 1.0;
            let size = 2.5 + Math::random() as f32 * 2.5;
            let color = [255, 180, 60]; // Orange explosion
            
            let particle = WebParticle::new(position, velocity, life, size, color);
            self.particles.push(particle);
        }
    }
    
    pub fn active_particle_count(&self) -> usize {
        self.particles.iter().filter(|p| p.active).count()
    }
    
    pub fn get_render_data(&self) -> Vec<f32> {
        let active_particles: Vec<&WebParticle> = self.particles.iter().filter(|p| p.active).collect();
        let mut data = Vec::with_capacity(active_particles.len() * 6);
        
        for particle in active_particles {
            data.extend_from_slice(&[
                particle.position.x,
                particle.position.y,
                particle.size,
                particle.color[0] as f32 / 255.0,
                particle.color[1] as f32 / 255.0,
                particle.color[2] as f32 / 255.0,
            ]);
        }
        
        data
    }
    
    pub fn clear(&mut self) {
        self.particles.clear();
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
    pub browser_info: BrowserInfo,
}

impl WebPerformanceMonitor {
    pub fn new() -> Self {
        let browser_info = BrowserInfo::detect();
        let initial_quality = if browser_info.is_low_end { 0 } else { browser_info.performance_tier };
        
        Self {
            last_frame_time: now(),
            frame_times: Vec::with_capacity(60),
            fps_counter: 0,
            fps_timer: 0.0,
            current_fps: 60.0,
            average_frame_time_ms: 16.67,
            quality_level: initial_quality,
            adaptive_quality: true,
            quality_cooldown: 0,
            browser_info,
        }
    }
    
    pub fn update(&mut self, current_time: f64) -> f32 {
        let frame_time = ((current_time - self.last_frame_time) / 1000.0) as f32;
        self.last_frame_time = current_time;
        
        // Cap delta time
        let capped_frame_time = frame_time.min(0.033); // Max 33ms
        
        // Track frame times for rolling average
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
                    self.quality_cooldown = 120; // 2 second cooldown at 60fps
                    console_log!("Quality reduced to {} (frame time: {:.2}ms)", 
                                self.quality_level, self.average_frame_time_ms);
                }
            } else if self.average_frame_time_ms < MAX_FRAME_TIME_MS * 0.7 {
                if self.quality_level < 2 {
                    self.quality_level += 1;
                    self.quality_cooldown = 300; // 5 second cooldown for increases
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
    pub mouse_pos: Vector2,
    pub mouse_delta: Vector2,
    pub touches: Vec<Vector2>,
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
        self.mouse_pos = Vector2::new(x, y);
        self.mouse_delta = Vector2::new(delta_x, delta_y);
    }
    
    pub fn set_touch(&mut self, touches: Vec<(f32, f32)>) {
        self.touches = touches.into_iter().map(|(x, y)| Vector2::new(x, y)).collect();
        self.touch_active = !self.touches.is_empty();
    }
    
    pub fn get_movement_input(&self) -> Vector2 {
        let mut movement = Vector2::zero();
        
        // Keyboard input (WASD + Arrow keys)
        if self.is_key_pressed(87) || self.is_key_pressed(38) { movement.y -= 1.0; } // W/Up
        if self.is_key_pressed(83) || self.is_key_pressed(40) { movement.y += 1.0; } // S/Down
        if self.is_key_pressed(65) || self.is_key_pressed(37) { movement.x -= 1.0; } // A/Left
        if self.is_key_pressed(68) || self.is_key_pressed(39) { movement.x += 1.0; } // D/Right
        
        // Normalize diagonal movement
        if movement.magnitude() > 1.0 {
            movement = movement.normalized();
        }
        
        movement
    }
}

// === WEB COLLISION SYSTEM ===

#[derive(Debug)]
pub struct WebCollisionSystem {
    spatial_grid: HashMap<(i32, i32), Vec<usize>>,
    cell_size: f32,
}

impl WebCollisionSystem {
    pub fn new() -> Self {
        Self {
            spatial_grid: HashMap::new(),
            cell_size: 64.0,
        }
    }
    
    pub fn update(&mut self, entities: &mut [WebEntity], particle_system: &mut WebParticleSystem) -> i32 {
        // Clear spatial grid
        self.spatial_grid.clear();
        
        // Populate spatial grid
        for (index, entity) in entities.iter().enumerate() {
            if !entity.active {
                continue;
            }
            
            let grid_x = (entity.position.x / self.cell_size) as i32;
            let grid_y = (entity.position.y / self.cell_size) as i32;
            
            self.spatial_grid.entry((grid_x, grid_y))
                .or_insert_with(Vec::new)
                .push(index);
        }
        
        let mut score_increment = 0;
        let mut collisions = Vec::new();
        
        // Check collisions within grid cells
        for entity_indices in self.spatial_grid.values() {
            for i in 0..entity_indices.len() {
                for j in (i + 1)..entity_indices.len() {
                    let idx_a = entity_indices[i];
                    let idx_b = entity_indices[j];
                    
                    let distance_sq = entities[idx_a].position.distance_squared_to(&entities[idx_b].position);
                    let collision_radius_sq = COLLISION_RADIUS * COLLISION_RADIUS;
                    
                    if distance_sq < collision_radius_sq {
                        collisions.push((idx_a, idx_b, distance_sq.sqrt()));
                        
                        // Score for player collisions
                        if entities[idx_a].entity_type == EntityType::Player || 
                           entities[idx_b].entity_type == EntityType::Player {
                            score_increment += 10;
                        }
                    }
                }
            }
        }
        
        // Resolve collisions
        for (idx_a, idx_b, distance) in collisions {
            let direction = (entities[idx_a].position - entities[idx_b].position).normalized();
            let overlap = COLLISION_RADIUS - distance;
            
            // Separate entities
            entities[idx_a].position += direction * overlap * 0.5;
            entities[idx_b].position -= direction * overlap * 0.5;
            
            // Apply collision response
            let bounce_force = 120.0;
            entities[idx_a].apply_force(direction * bounce_force);
            entities[idx_b].apply_force(direction * -bounce_force);
            
            // Create particle effect
            let collision_point = (entities[idx_a].position + entities[idx_b].position) * 0.5;
            particle_system.create_explosion(collision_point, 3);
        }
        
        score_increment
    }
}

// === MAIN WEB GAME STATE ===

#[wasm_bindgen]
pub struct WebGameState {
    entities: Vec<WebEntity>,
    particle_system: WebParticleSystem,
    collision_system: WebCollisionSystem,
    input_system: WebInputSystem,
    performance: WebPerformanceMonitor,
    
    // Camera
    camera_x: f32,
    camera_y: f32,
    camera_shake: f32,
    
    // Game state
    score: i32,
    level: i32,
    paused: bool,
    debug_mode: bool,
}

#[wasm_bindgen]
impl WebGameState {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WebGameState {
        console_log!("Creating Web Game State for v1.8 Classic Edition");
        
        let performance = WebPerformanceMonitor::new();
        console_log!("Detected: {} cores, {} tier, mobile: {}", 
                    performance.browser_info.cpu_cores,
                    performance.browser_info.performance_tier,
                    performance.browser_info.is_mobile);
        
        let mut game_state = WebGameState {
            entities: Vec::with_capacity(MAX_ENTITIES),
            particle_system: WebParticleSystem::new(MAX_PARTICLES),
            collision_system: WebCollisionSystem::new(),
            input_system: WebInputSystem::new(),
            performance,
            
            camera_x: CANVAS_WIDTH / 2.0,
            camera_y: CANVAS_HEIGHT / 2.0,
            camera_shake: 0.0,
            
            score: 0,
            level: 1,
            paused: false,
            debug_mode: false,
        };
        
        game_state.initialize_entities();
        game_state
    }
    
    fn initialize_entities(&mut self) {
        // Create player entity
        let player = WebEntity::new(
            Vector2::new(CANVAS_WIDTH / 2.0, CANVAS_HEIGHT / 2.0),
            0,
            "Player".to_string(),
            EntityType::Player,
        );
        self.entities.push(player);
        
        // Create environment entities (fewer for mobile)
        let entity_count = if self.performance.browser_info.is_mobile { 12 } else { 18 };
        
        for i in 0..entity_count {
            let position = Vector2::new(
                50.0 + (CANVAS_WIDTH - 100.0) * Math::random() as f32,
                50.0 + (CANVAS_HEIGHT - 100.0) * Math::random() as f32,
            );
            
            let texture_id = 1 + (Math::random() * 3.0) as u32;
            let name = format!("Object_{}", i);
            
            let mut env_entity = WebEntity::new(position, texture_id, name, EntityType::Environment);
            
            // Give some initial velocity for dynamic gameplay
            env_entity.velocity = Vector2::new(
                (Math::random() as f32 - 0.5) * 40.0,
                (Math::random() as f32 - 0.5) * 40.0,
            );
            
            self.entities.push(env_entity);
        }
        
        console_log!("Initialized {} entities for web", self.entities.len());
    }
    
    #[wasm_bindgen]
    pub fn update(&mut self, current_time: f64) {
        if self.paused {
            return;
        }
        
        let delta_time = self.performance.update(current_time);
        
        // Process input
        let movement = self.input_system.get_movement_input();
        if movement.magnitude() > 0.1 {
            if let Some(player) = self.entities.first_mut() {
                let move_speed = 180.0; // Responsive web movement
                player.apply_force(movement * move_speed * delta_time);
            }
        }
        
        // Touch input for mobile
        if self.input_system.touch_active {
            if let (Some(player), Some(touch)) = (self.entities.first_mut(), self.input_system.touches.first()) {
                let direction = *touch - player.position;
                let distance = direction.magnitude();
                
                if distance > 32.0 { // Dead zone
                    let touch_force = direction.normalized() * 120.0;
                    player.apply_force(touch_force * delta_time);
                }
            }
        }
        
        // Update entities
        for entity in &mut self.entities {
            entity.update(delta_time);
            
            // Simple AI for non-player entities
            if entity.entity_type != EntityType::Player && entity.active {
                let time_factor = current_time * 0.0008;
                let pattern = (time_factor + entity.texture_id as f64).sin() as f32;
                let ai_force = Vector2::new(
                    (time_factor.sin() as f32) * pattern * 30.0,
                    (time_factor.cos() as f32 * 1.3) * pattern * 30.0,
                );
                entity.apply_force(ai_force * delta_time * 0.1);
            }
        }
        
        // Update systems based on quality level
        if self.performance.quality_level >= 1 {
            let score_increment = self.collision_system.update(&mut self.entities, &mut self.particle_system);
            self.score += score_increment;
            
            if score_increment > 0 {
                self.camera_shake = 5.0; // Screen shake on collision
            }
        }
        
        if self.performance.quality_level >= 2 {
            self.particle_system.update(delta_time);
        }
        
        // Update camera with smooth following and shake
        if let Some(player) = self.entities.first() {
            let target_x = player.position.x;
            let target_y = player.position.y;
            
            let lerp_speed = 3.0 * delta_time;
            self.camera_x += (target_x - self.camera_x) * lerp_speed;
            self.camera_y += (target_y - self.camera_y) * lerp_speed;
            
            // Camera shake
            if self.camera_shake > 0.0 {
                self.camera_x += (Math::random() as f32 - 0.5) * self.camera_shake;
                self.camera_y += (Math::random() as f32 - 0.5) * self.camera_shake;
                self.camera_shake *= 0.9; // Decay
            }
        }
        
        // Cleanup dead entities
        self.entities.retain(|e| e.is_alive());
        
        // Debug output
        if self.debug_mode && self.performance.fps_counter % 60 == 0 {
            console_log!("FPS: {:.1}, Entities: {}, Particles: {}, Quality: {}", 
                        self.performance.current_fps, 
                        self.entities.len(), 
                        self.particle_system.active_particle_count(),
                        self.performance.quality_level);
        }
    }
    
    // === WASM EXPORTS ===
    
    #[wasm_bindgen]
    pub fn handle_key_event(&mut self, key_code: u32, pressed: bool) {
        self.input_system.set_key(key_code, pressed);
        
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
                82 => { // R key
                    self.reset_game();
                }
                _ => {}
            }
        }
    }
    
    #[wasm_bindgen]
    pub fn handle_mouse_event(&mut self, x: f32, y: f32, delta_x: f32, delta_y: f32) {
        self.input_system.set_mouse(x, y, delta_x, delta_y);
    }
    
    #[wasm_bindgen]
    pub fn handle_touch_event(&mut self, touches: Vec<f32>) {
        let touch_pairs: Vec<(f32, f32)> = touches
            .chunks_exact(2)
            .map(|chunk| (chunk[0], chunk[1]))
            .collect();
        self.input_system.set_touch(touch_pairs);
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
        self.particle_system.active_particle_count()
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
    pub fn is_paused(&self) -> bool {
        self.paused
    }
    
    #[wasm_bindgen]
    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }
    
    #[wasm_bindgen]
    pub fn get_camera_position(&self) -> Vec<f32> {
        vec![self.camera_x, self.camera_y]
    }
    
    #[wasm_bindgen]
    pub fn get_entity_render_data(&self) -> Vec<f32> {
        let mut data = Vec::with_capacity(self.entities.len() * 8);
        
        for entity in &self.entities {
            if !entity.active {
                continue;
            }
            
            // Entity render data: pos_x, pos_y, vel_x, vel_y, rotation, texture_id, health_ratio, is_player
            data.extend_from_slice(&[
                entity.position.x,
                entity.position.y,
                entity.velocity.x,
                entity.velocity.y,
                entity.rotation,
                entity.texture_id as f32,
                entity.health as f32 / entity.max_health as f32,
                if entity.entity_type == EntityType::Player { 1.0 } else { 0.0 },
            ]);
        }
        
        data
    }
    
    #[wasm_bindgen]
    pub fn get_particle_render_data(&self) -> Vec<f32> {
        self.particle_system.get_render_data()
    }
    
    #[wasm_bindgen]
    pub fn create_explosion(&mut self, x: f32, y: f32, count: usize) {
        let position = Vector2::new(x, y);
        self.particle_system.create_explosion(position, count);
        self.camera_shake = 8.0; // Add screen shake
    }
    
    #[wasm_bindgen]
    pub fn add_entity(&mut self, x: f32, y: f32, texture_id: u32, name: String, entity_type: String) -> bool {
        if self.entities.len() >= MAX_ENTITIES {
            return false;
        }
        
        let etype = match entity_type.as_str() {
            "Player" => EntityType::Player,
            "Environment" => EntityType::Environment,
            "Pickup" => EntityType::Pickup,
            "Projectile" => EntityType::Projectile,
            _ => EntityType::Environment,
        };
        
        let entity = WebEntity::new(Vector2::new(x, y), texture_id, name, etype);
        self.entities.push(entity);
        true
    }
    
    #[wasm_bindgen]
    pub fn reset_game(&mut self) {
        console_log!("Resetting web game state");
        
        self.entities.clear();
        self.particle_system.clear();
        self.score = 0;
        self.level = 1;
        self.paused = false;
        self.camera_shake = 0.0;
        
        // Reset camera
        self.camera_x = CANVAS_WIDTH / 2.0;
        self.camera_y = CANVAS_HEIGHT / 2.0;
        
        // Reset performance metrics
        self.performance.fps_counter = 0;
        self.performance.fps_timer = 0.0;
        self.performance.quality_cooldown = 0;
        
        self.initialize_entities();
    }
    
    #[wasm_bindgen]
    pub fn get_browser_info(&self) -> JsValue {
        let info = js_sys::Object::new();
        
        js_sys::Reflect::set(&info, &"isMobile".into(), &self.performance.browser_info.is_mobile.into()).unwrap();
        js_sys::Reflect::set(&info, &"isLowEnd".into(), &self.performance.browser_info.is_low_end.into()).unwrap();
        js_sys::Reflect::set(&info, &"cpuCores".into(), &self.performance.browser_info.cpu_cores.into()).unwrap();
        js_sys::Reflect::set(&info, &"performanceTier".into(), &self.performance.browser_info.performance_tier.into()).unwrap();
        js_sys::Reflect::set(&info, &"supportsWebGL2".into(), &self.performance.browser_info.supports_webgl2.into()).unwrap();
        
        info.into()
    }
    
    #[wasm_bindgen]
    pub fn get_performance_info(&self) -> JsValue {
        let info = js_sys::Object::new();
        
        js_sys::Reflect::set(&info, &"fps".into(), &self.performance.current_fps.into()).unwrap();
        js_sys::Reflect::set(&info, &"frameTime".into(), &self.performance.average_frame_time_ms.into()).unwrap();
        js_sys::Reflect::set(&info, &"qualityLevel".into(), &self.performance.quality_level.into()).unwrap();
        js_sys::Reflect::set(&info, &"adaptiveQuality".into(), &self.performance.adaptive_quality.into()).unwrap();
        js_sys::Reflect::set(&info, &"entityCount".into(), &self.entities.len().into()).unwrap();
        js_sys::Reflect::set(&info, &"particleCount".into(), &self.particle_system.active_particle_count().into()).unwrap();
        js_sys::Reflect::set(&info, &"isPerformanceGood".into(), &self.performance.is_performance_good().into()).unwrap();
        
        info.into()
    }
    
    #[wasm_bindgen]
    pub fn cleanup(&mut self) {
        console_log!("Cleaning up Web Game Engine v1.8");
        self.entities.clear();
        self.particle_system.clear();
    }
}

// === WEB ENGINE WRAPPER ===

#[wasm_bindgen]
pub struct WebGameEngine {
    game_state: WebGameState,
}

#[wasm_bindgen]
impl WebGameEngine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WebGameEngine {
        console_log!("Initializing WASM Web Game Engine v1.8 Classic");
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
        
        // Entity data
        let entities = self.game_state.get_entity_render_data();
        let entity_array = js_sys::Float32Array::from(&entities[..]);
        js_sys::Reflect::set(&data, &"entities".into(), &entity_array.into()).unwrap();
        
        // Particle data
        let particles = self.game_state.get_particle_render_data();
        let particle_array = js_sys::Float32Array::from(&particles[..]);
        js_sys::Reflect::set(&data, &"particles".into(), &particle_array.into()).unwrap();
        
        // Camera data
        let camera = self.game_state.get_camera_position();
        let camera_array = js_sys::Float32Array::from(&camera[..]);
        js_sys::Reflect::set(&data, &"camera".into(), &camera_array.into()).unwrap();
        
        // Canvas dimensions
        js_sys::Reflect::set(&data, &"canvasWidth".into(), &CANVAS_WIDTH.into()).unwrap();
        js_sys::Reflect::set(&data, &"canvasHeight".into(), &CANVAS_HEIGHT.into()).unwrap();
        
        data.into()
    }
    
    #[wasm_bindgen]
    pub fn get_game_info(&self) -> JsValue {
        let info = js_sys::Object::new();
        
        js_sys::Reflect::set(&info, &"score".into(), &self.game_state.get_score().into()).unwrap();
        js_sys::Reflect::set(&info, &"entityCount".into(), &self.game_state.get_entity_count().into()).unwrap();
        js_sys::Reflect::set(&info, &"particleCount".into(), &self.game_state.get_particle_count().into()).unwrap();
        js_sys::Reflect::set(&info, &"paused".into(), &self.game_state.is_paused().into()).unwrap();
        
        info.into()
    }
    
    #[wasm_bindgen]
    pub fn get_performance_info(&self) -> JsValue {
        self.game_state.get_performance_info()
    }
    
    #[wasm_bindgen]
    pub fn get_browser_info(&self) -> JsValue {
        self.game_state.get_browser_info()
    }
    
    #[wasm_bindgen]
    pub fn set_quality(&mut self, quality: u8) {
        self.game_state.set_quality_level(quality);
    }
    
    #[wasm_bindgen]
    pub fn enable_adaptive_quality(&mut self, enabled: bool) {
        self.game_state.enable_adaptive_quality(enabled);
    }
    
    #[wasm_bindgen]
    pub fn create_explosion(&mut self, x: f32, y: f32, count: usize) {
        self.game_state.create_explosion(x, y, count);
    }
    
    #[wasm_bindgen]
    pub fn add_entity(&mut self, x: f32, y: f32, texture_id: u32, name: String, entity_type: String) -> bool {
        self.game_state.add_entity(x, y, texture_id, name, entity_type)
    }
    
    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.game_state.reset_game();
    }
    
    #[wasm_bindgen]
    pub fn cleanup(&mut self) {
        self.game_state.cleanup();
    }
    
    #[wasm_bindgen]
    pub fn toggle_pause(&mut self) {
        self.game_state.toggle_pause();
    }
    
    #[wasm_bindgen]
    pub fn is_paused(&self) -> bool {
        self.game_state.is_paused()
    }
}

// === UTILITY FUNCTIONS FOR WEB INTEGRATION ===

#[wasm_bindgen]
pub fn get_optimal_quality_for_device() -> u8 {
    let browser_info = BrowserInfo::detect();
    
    if browser_info.is_mobile || browser_info.cpu_cores <= 2 {
        0 // Low quality for mobile/weak devices
    } else if browser_info.cpu_cores <= 4 {
        1 // Medium quality for average devices
    } else {
        2 // High quality for powerful devices
    }
}

#[wasm_bindgen]
pub fn detect_browser_capabilities() -> JsValue {
    let info = BrowserInfo::detect();
    let caps = js_sys::Object::new();
    
    js_sys::Reflect::set(&caps, &"isMobile".into(), &info.is_mobile.into()).unwrap();
    js_sys::Reflect::set(&caps, &"isLowEnd".into(), &info.is_low_end.into()).unwrap();
    js_sys::Reflect::set(&caps, &"cpuCores".into(), &info.cpu_cores.into()).unwrap();
    js_sys::Reflect::set(&caps, &"performanceTier".into(), &info.performance_tier.into()).unwrap();
    js_sys::Reflect::set(&caps, &"supportsWebGL2".into(), &info.supports_webgl2.into()).unwrap();
    js_sys::Reflect::set(&caps, &"recommendedQuality".into(), &get_optimal_quality_for_device().into()).unwrap();
    
    caps.into()
}

// === ENTRY POINT ===

#[wasm_bindgen(start)]
pub fn main() {
    console_log!("WASM Web Game Engine v1.8 Classic Edition loaded successfully!");
    console_log!("Features: Web-Optimized, Adaptive Quality, Mobile Support, Browser Detection");
    
    // Detect and log browser capabilities
    let browser_info = BrowserInfo::detect();
    console_log!("Browser detected: {} cores, tier {}, mobile: {}", 
                browser_info.cpu_cores, 
                browser_info.performance_tier, 
                browser_info.is_mobile);
}
