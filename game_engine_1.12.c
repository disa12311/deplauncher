#include <emscripten.h>
#include <emscripten/html5.h>
#include <emscripten/threading.h>
#include <emscripten/fetch.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>
#include <stdbool.h>
#include <time.h>

// Advanced Game Engine for Deplauncher 1.12 - Enhanced Edition
// Cleaned and optimized version with improved architecture

// === CONSTANTS ===
#define MAX_ENTITIES 5000
#define MAX_PARTICLES 10000
#define MAX_LIGHTS 50
#define CANVAS_WIDTH 1920
#define CANVAS_HEIGHT 1080
#define PHYSICS_SUBSTEPS 4
#define NETWORK_BUFFER_SIZE 8192
#define ENTITY_NAME_SIZE 64
#define ENTITY_TAG_SIZE 32

// === CORE STRUCTURES ===

// 3D Vector for better code organization
typedef struct {
    float x, y, z;
} Vector3;

// Transform component
typedef struct {
    Vector3 position;
    Vector3 rotation;
    Vector3 scale;
    Vector3 velocity;
    Vector3 acceleration;
} Transform;

// Physics component
typedef struct {
    float mass;
    float friction;
    float bounciness;
    float drag;
    bool is_kinematic;
    bool has_gravity;
} Physics;

// Rendering component
typedef struct {
    int texture_id;
    int normal_map_id;
    int specular_map_id;
    float color[4];
    float metallic;
    float roughness;
    bool cast_shadows;
    bool receive_shadows;
} Renderer;

// Game logic component
typedef struct {
    bool active;
    int health;
    int max_health;
    float energy;
    float max_energy;
    char name[ENTITY_NAME_SIZE];
    char tag[ENTITY_TAG_SIZE];
    int layer;
} GameLogic;

// AI component
typedef struct {
    int state;
    float timer;
    Vector3 target_position;
    int target_entity_id;
} AI;

// Advanced entity with component-based architecture
typedef struct {
    Transform transform;
    Physics physics;
    Renderer renderer;
    GameLogic logic;
    AI ai;
    
    // Animation system
    int current_animation;
    float animation_time;
    float animation_speed;
    bool animation_loop;
    
    // Networking
    bool networked;
    int owner_id;
    float last_sync_time;
} AdvancedEntity;

// Particle system
typedef struct {
    Vector3 position;
    Vector3 velocity;
    float color[4];
    float life;
    float max_life;
    float size;
    float rotation;
    bool active;
} Particle;

// Lighting system
typedef struct {
    Vector3 position;
    Vector3 direction;
    float color[3];
    float intensity;
    float range;
    int type; // 0=directional, 1=point, 2=spot
    float spot_angle;
    bool cast_shadows;
    bool active;
} Light;

// Performance metrics
typedef struct {
    double last_frame_time;
    int fps_counter;
    double fps_timer;
    float frame_time_ms;
    int draw_calls;
} PerformanceMetrics;

// Graphics settings
typedef struct {
    bool bloom_enabled;
    bool ssao_enabled;
    bool motion_blur_enabled;
    bool pbr_enabled;
    bool shadows_enabled;
    int shadow_quality;
    bool reflections_enabled;
    float exposure;
    float gamma;
} GraphicsSettings;

// Camera system
typedef struct {
    Vector3 position;
    Vector3 rotation;
    float fov;
    float near_plane;
    float far_plane;
} Camera;

// Physics settings
typedef struct {
    Vector3 gravity;
    float air_density;
    bool enabled;
} PhysicsSettings;

// Audio settings
typedef struct {
    float master_volume;
    float sfx_volume;
    float music_volume;
} AudioSettings;

// Network settings
typedef struct {
    bool multiplayer_enabled;
    int player_id;
    char server_url[256];
    char buffer[NETWORK_BUFFER_SIZE];
} NetworkSettings;

// Main game state - better organized
typedef struct {
    // Entity system
    AdvancedEntity entities[MAX_ENTITIES];
    int entity_count;
    
    // Particle system
    Particle particles[MAX_PARTICLES];
    int particle_count;
    
    // Lighting system
    Light lights[MAX_LIGHTS];
    int light_count;
    
    // Core systems
    Camera camera;
    PhysicsSettings physics;
    GraphicsSettings graphics;
    AudioSettings audio;
    NetworkSettings network;
    PerformanceMetrics performance;
    
    // Game state
    int score;
    int level;
    float time_scale;
    bool paused;
} AdvancedGameState;

// === GLOBAL STATE ===
static AdvancedGameState* g_game_state = NULL;
static bool g_initialized = false;

// === UTILITY FUNCTIONS ===

// Vector3 operations
Vector3 vec3_create(float x, float y, float z) {
    Vector3 v = {x, y, z};
    return v;
}

Vector3 vec3_add(Vector3 a, Vector3 b) {
    return vec3_create(a.x + b.x, a.y + b.y, a.z + b.z);
}

Vector3 vec3_subtract(Vector3 a, Vector3 b) {
    return vec3_create(a.x - b.x, a.y - b.y, a.z - b.z);
}

Vector3 vec3_multiply_scalar(Vector3 v, float scalar) {
    return vec3_create(v.x * scalar, v.y * scalar, v.z * scalar);
}

float vec3_magnitude(Vector3 v) {
    return sqrtf(v.x * v.x + v.y * v.y + v.z * v.z);
}

Vector3 vec3_normalize(Vector3 v) {
    float mag = vec3_magnitude(v);
    if (mag > 0.001f) {
        return vec3_multiply_scalar(v, 1.0f / mag);
    }
    return vec3_create(0, 0, 0);
}

// === SYSTEM FUNCTIONS ===

// Memory management
bool allocate_game_state() {
    g_game_state = (AdvancedGameState*)calloc(1, sizeof(AdvancedGameState));
    return g_game_state != NULL;
}

void deallocate_game_state() {
    if (g_game_state) {
        free(g_game_state);
        g_game_state = NULL;
    }
}

// Entity management
AdvancedEntity* create_entity(Vector3 position, const char* name) {
    if (!g_game_state || g_game_state->entity_count >= MAX_ENTITIES) {
        return NULL;
    }
    
    AdvancedEntity* entity = &g_game_state->entities[g_game_state->entity_count++];
    
    // Initialize transform
    entity->transform.position = position;
    entity->transform.rotation = vec3_create(0, 0, 0);
    entity->transform.scale = vec3_create(1, 1, 1);
    entity->transform.velocity = vec3_create(0, 0, 0);
    entity->transform.acceleration = vec3_create(0, 0, 0);
    
    // Initialize physics
    entity->physics.mass = 1.0f;
    entity->physics.friction = 0.1f;
    entity->physics.bounciness = 0.5f;
    entity->physics.drag = 0.01f;
    entity->physics.is_kinematic = false;
    entity->physics.has_gravity = true;
    
    // Initialize renderer
    entity->renderer.texture_id = 0;
    entity->renderer.normal_map_id = -1;
    entity->renderer.specular_map_id = -1;
    entity->renderer.color[0] = entity->renderer.color[1] = entity->renderer.color[2] = entity->renderer.color[3] = 1.0f;
    entity->renderer.metallic = 0.0f;
    entity->renderer.roughness = 0.5f;
    entity->renderer.cast_shadows = false;
    entity->renderer.receive_shadows = true;
    
    // Initialize game logic
    entity->logic.active = true;
    entity->logic.health = 100;
    entity->logic.max_health = 100;
    entity->logic.energy = 100.0f;
    entity->logic.max_energy = 100.0f;
    strncpy(entity->logic.name, name, ENTITY_NAME_SIZE - 1);
    strcpy(entity->logic.tag, "Untagged");
    entity->logic.layer = 0;
    
    // Initialize AI
    entity->ai.state = 0;
    entity->ai.timer = 0.0f;
    entity->ai.target_position = vec3_create(0, 0, 0);
    entity->ai.target_entity_id = -1;
    
    // Initialize animation
    entity->current_animation = 0;
    entity->animation_time = 0.0f;
    entity->animation_speed = 1.0f;
    entity->animation_loop = true;
    
    // Initialize networking
    entity->networked = false;
    entity->owner_id = -1;
    entity->last_sync_time = 0.0f;
    
    return entity;
}

// Physics system
void update_physics_system(float delta_time) {
    if (!g_game_state->physics.enabled) return;
    
    float sub_delta = delta_time / PHYSICS_SUBSTEPS;
    
    for (int step = 0; step < PHYSICS_SUBSTEPS; step++) {
        for (int i = 0; i < g_game_state->entity_count; i++) {
            AdvancedEntity* entity = &g_game_state->entities[i];
            if (!entity->logic.active || entity->physics.is_kinematic) continue;
            
            // Apply gravity
            if (entity->physics.has_gravity) {
                entity->transform.acceleration = vec3_add(entity->transform.acceleration, g_game_state->physics.gravity);
            }
            
            // Apply drag
            float speed = vec3_magnitude(entity->transform.velocity);
            if (speed > 0.01f) {
                Vector3 drag_direction = vec3_normalize(entity->transform.velocity);
                float drag_force = 0.5f * g_game_state->physics.air_density * speed * speed * entity->physics.drag;
                Vector3 drag_acceleration = vec3_multiply_scalar(drag_direction, -drag_force / entity->physics.mass);
                entity->transform.acceleration = vec3_add(entity->transform.acceleration, drag_acceleration);
            }
            
            // Integration
            entity->transform.velocity = vec3_add(entity->transform.velocity, 
                                                vec3_multiply_scalar(entity->transform.acceleration, sub_delta));
            entity->transform.position = vec3_add(entity->transform.position, 
                                                vec3_multiply_scalar(entity->transform.velocity, sub_delta));
            
            // Apply friction
            entity->transform.velocity = vec3_multiply_scalar(entity->transform.velocity, 
                                                            1.0f - entity->physics.friction * sub_delta);
            
            // Reset acceleration
            entity->transform.acceleration = vec3_create(0, 0, 0);
        }
    }
}

// Particle system
void init_particle_system() {
    g_game_state->particle_count = 0;
    memset(g_game_state->particles, 0, sizeof(g_game_state->particles));
}

void create_particle_explosion(Vector3 position, int count) {
    for (int i = 0; i < count && g_game_state->particle_count < MAX_PARTICLES; i++) {
        Particle* p = &g_game_state->particles[g_game_state->particle_count++];
        
        // Random explosion direction
        float angle_xz = ((float)rand() / RAND_MAX) * 2.0f * M_PI;
        float angle_y = ((float)rand() / RAND_MAX) * M_PI - M_PI / 2.0f;
        float speed = 100.0f + ((float)rand() / RAND_MAX) * 200.0f;
        
        p->position = position;
        p->velocity = vec3_create(
            cosf(angle_xz) * cosf(angle_y) * speed,
            sinf(angle_y) * speed,
            sinf(angle_xz) * cosf(angle_y) * speed
        );
        
        p->color[0] = 1.0f;
        p->color[1] = 0.5f + ((float)rand() / RAND_MAX) * 0.5f;
        p->color[2] = 0.0f;
        p->color[3] = 1.0f;
        
        p->life = p->max_life = 1.0f + ((float)rand() / RAND_MAX) * 2.0f;
        p->size = 2.0f + ((float)rand() / RAND_MAX) * 4.0f;
        p->rotation = 0.0f;
        p->active = true;
    }
}

void update_particle_system(float delta_time) {
    for (int i = 0; i < g_game_state->particle_count; i++) {
        Particle* p = &g_game_state->particles[i];
        if (!p->active) continue;
        
        // Update physics
        p->velocity = vec3_add(p->velocity, vec3_multiply_scalar(g_game_state->physics.gravity, delta_time));
        p->position = vec3_add(p->position, vec3_multiply_scalar(p->velocity, delta_time));
        p->rotation += delta_time * 180.0f;
        
        // Update life
        p->life -= delta_time;
        if (p->life <= 0.0f) {
            p->active = false;
        }
        
        // Update appearance based on life
        float life_ratio = p->life / p->max_life;
        p->color[3] = life_ratio;
        p->size *= 0.995f;
    }
    
    // Cleanup inactive particles
    int write_index = 0;
    for (int read_index = 0; read_index < g_game_state->particle_count; read_index++) {
        if (g_game_state->particles[read_index].active) {
            if (write_index != read_index) {
                g_game_state->particles[write_index] = g_game_state->particles[read_index];
            }
            write_index++;
        }
    }
    g_game_state->particle_count = write_index;
}

// AI system
void update_ai_system(float delta_time) {
    for (int i = 0; i < g_game_state->entity_count; i++) {
        AdvancedEntity* entity = &g_game_state->entities[i];
        if (!entity->logic.active) continue;
        
        entity->ai.timer -= delta_time;
        
        if (entity->ai.timer <= 0.0f) {
            entity->ai.timer = 0.5f; // Decision every 0.5 seconds
            
            // Simple AI behaviors
            if (strcmp(entity->logic.tag, "Environment") == 0) {
                // Wander behavior
                entity->ai.target_position = vec3_create(
                    ((float)rand() / RAND_MAX) * CANVAS_WIDTH,
                    ((float)rand() / RAND_MAX) * CANVAS_HEIGHT,
                    entity->transform.position.z
                );
            }
        }
        
        // Execute AI behavior
        Vector3 direction = vec3_subtract(entity->ai.target_position, entity->transform.position);
        float distance = vec3_magnitude(direction);
        
        if (distance > 5.0f) {
            Vector3 move_force = vec3_multiply_scalar(vec3_normalize(direction), 100.0f);
            entity->transform.acceleration = vec3_add(entity->transform.acceleration, 
                                                    vec3_multiply_scalar(move_force, 1.0f / entity->physics.mass));
        }
    }
}

// Collision detection
void update_collision_system() {
    for (int i = 0; i < g_game_state->entity_count; i++) {
        for (int j = i + 1; j < g_game_state->entity_count; j++) {
            AdvancedEntity* a = &g_game_state->entities[i];
            AdvancedEntity* b = &g_game_state->entities[j];
            
            if (!a->logic.active || !b->logic.active) continue;
            
            Vector3 direction = vec3_subtract(a->transform.position, b->transform.position);
            float distance = vec3_magnitude(direction);
            
            if (distance < 32.0f) { // Collision threshold
                // Separation
                float overlap = 32.0f - distance;
                Vector3 separation = vec3_multiply_scalar(vec3_normalize(direction), overlap * 0.5f);
                
                a->transform.position = vec3_add(a->transform.position, separation);
                b->transform.position = vec3_subtract(b->transform.position, separation);
                
                // Collision response
                float bounce_force = 150.0f * (a->physics.bounciness + b->physics.bounciness) * 0.5f;
                Vector3 bounce_impulse = vec3_multiply_scalar(vec3_normalize(direction), bounce_force);
                
                a->transform.velocity = vec3_add(a->transform.velocity, bounce_impulse);
                b->transform.velocity = vec3_subtract(b->transform.velocity, bounce_impulse);
                
                // Create particle effect
                Vector3 collision_point = vec3_multiply_scalar(vec3_add(a->transform.position, b->transform.position), 0.5f);
                create_particle_explosion(collision_point, 3);
                
                // Update score
                if (strcmp(a->logic.tag, "Player") == 0 || strcmp(b->logic.tag, "Player") == 0) {
                    g_game_state->score += 5;
                }
            }
        }
    }
}

// Cleanup system
void cleanup_entities() {
    int write_index = 0;
    for (int read_index = 0; read_index < g_game_state->entity_count; read_index++) {
        if (g_game_state->entities[read_index].logic.active) {
            if (write_index != read_index) {
                g_game_state->entities[write_index] = g_game_state->entities[read_index];
            }
            write_index++;
        }
    }
    g_game_state->entity_count = write_index;
}

// === MAIN SYSTEMS ===

// Initialization
void init_advanced_engine() {
    if (!allocate_game_state()) {
        printf("Failed to allocate game state memory\n");
        return;
    }
    
    // Initialize camera
    g_game_state->camera.position = vec3_create(CANVAS_WIDTH / 2.0f, CANVAS_HEIGHT / 2.0f, -500.0f);
    g_game_state->camera.rotation = vec3_create(0, 0, 0);
    g_game_state->camera.fov = 75.0f;
    g_game_state->camera.near_plane = 0.1f;
    g_game_state->camera.far_plane = 1000.0f;
    
    // Initialize physics
    g_game_state->physics.gravity = vec3_create(0.0f, -980.0f, 0.0f);
    g_game_state->physics.air_density = 1.225f;
    g_game_state->physics.enabled = true;
    
    // Initialize graphics settings
    g_game_state->graphics.bloom_enabled = true;
    g_game_state->graphics.ssao_enabled = true;
    g_game_state->graphics.motion_blur_enabled = false;
    g_game_state->graphics.pbr_enabled = true;
    g_game_state->graphics.shadows_enabled = true;
    g_game_state->graphics.shadow_quality = 2;
    g_game_state->graphics.reflections_enabled = true;
    g_game_state->graphics.exposure = 1.0f;
    g_game_state->graphics.gamma = 2.2f;
    
    // Initialize audio
    g_game_state->audio.master_volume = 1.0f;
    g_game_state->audio.sfx_volume = 0.8f;
    g_game_state->audio.music_volume = 0.6f;
    
    // Initialize networking
    g_game_state->network.multiplayer_enabled = false;
    g_game_state->network.player_id = 0;
    strcpy(g_game_state->network.server_url, "");
    
    // Initialize game state
    g_game_state->score = 0;
    g_game_state->level = 1;
    g_game_state->time_scale = 1.0f;
    g_game_state->paused = false;
    g_game_state->performance.last_frame_time = emscripten_get_now();
    
    // Initialize particle system
    init_particle_system();
    
    // Create player entity
    AdvancedEntity* player = create_entity(vec3_create(CANVAS_WIDTH / 2.0f, CANVAS_HEIGHT / 2.0f, 0.0f), "Player");
    if (player) {
        player->physics.has_gravity = false; // Top-down view
        strcpy(player->logic.tag, "Player");
        player->renderer.cast_shadows = true;
        player->renderer.color[0] = 0.2f;
        player->renderer.color[1] = 0.8f;
        player->renderer.color[2] = 1.0f;
    }
    
    // Create environment entities
    for (int i = 0; i < 50; i++) {
        Vector3 pos = vec3_create(
            (float)(rand() % (int)CANVAS_WIDTH),
            (float)(rand() % (int)CANVAS_HEIGHT),
            ((float)(rand() % 200) - 100)
        );
        
        AdvancedEntity* env = create_entity(pos, "Environment");
        if (env) {
            snprintf(env->logic.name, ENTITY_NAME_SIZE, "Obj_%d", i);
            env->physics.mass = 0.5f + ((float)rand() / RAND_MAX) * 2.0f;
            env->renderer.metallic = (float)rand() / RAND_MAX;
            env->renderer.roughness = 0.2f + ((float)rand() / RAND_MAX) * 0.8f;
            env->renderer.color[0] = 0.5f + ((float)rand() / RAND_MAX) * 0.5f;
            env->renderer.color[1] = 0.5f + ((float)rand() / RAND_MAX) * 0.5f;
            env->renderer.color[2] = 0.5f + ((float)rand() / RAND_MAX) * 0.5f;
            strcpy(env->logic.tag, "Environment");
        }
    }
    
    g_initialized = true;
    printf("Advanced Game Engine v1.12 initialized successfully\n");
    printf("Entities: %d\n", g_game_state->entity_count);
}

// Main update function
void update_advanced_game_logic(double current_time) {
    if (g_game_state->paused) return;
    
    double frame_start = emscripten_get_now();
    float delta_time = (float)(current_time - g_game_state->performance.last_frame_time) / 1000.0f * g_game_state->time_scale;
    g_game_state->performance.last_frame_time = current_time;
    
    // Cap delta time
    if (delta_time > 0.033f) delta_time = 0.033f;
    
    // Update FPS counter
    g_game_state->performance.fps_counter++;
    g_game_state->performance.fps_timer += delta_time;
    if (g_game_state->performance.fps_timer >= 1.0) {
        printf("FPS: %d, Frame Time: %.2fms\n", 
               g_game_state->performance.fps_counter, g_game_state->performance.frame_time_ms);
        g_game_state->performance.fps_counter = 0;
        g_game_state->performance.fps_timer = 0.0;
    }
    
    // Update all systems
    update_physics_system(delta_time);
    update_ai_system(delta_time);
    update_particle_system(delta_time);
    update_collision_system();
    cleanup_entities();
    
    // Update camera to follow player
    if (g_game_state->entity_count > 0) {
        AdvancedEntity* player = &g_game_state->entities[0];
        float lerp_speed = 5.0f * delta_time;
        g_game_state->camera.position.x += (player->transform.position.x - g_game_state->camera.position.x) * lerp_speed;
        g_game_state->camera.position.y += (player->transform.position.y - g_game_state->camera.position.y) * lerp_speed;
    }
    
    // Calculate frame time
    double frame_end = emscripten_get_now();
    g_game_state->performance.frame_time_ms = (float)(frame_end - frame_start);
}

// Input handling
void handle_advanced_input(int key_code, bool pressed) {
    if (g_game_state->entity_count == 0) return;
    
    AdvancedEntity* player = &g_game_state->entities[0];
    float move_speed = 300.0f;
    
    if (pressed) {
        switch (key_code) {
            case 87: // W
                player->transform.acceleration.y -= move_speed;
                break;
            case 83: // S
                player->transform.acceleration.y += move_speed;
                break;
            case 65: // A
                player->transform.acceleration.x -= move_speed;
                break;
            case 68: // D
                player->transform.acceleration.x += move_speed;
                break;
            case 32: // Space
                g_game_state->paused = !g_game_state->paused;
                break;
        }
    }
}

// === WASM EXPORTS ===

EMSCRIPTEN_KEEPALIVE
int wasm_init_advanced_game() {
    printf("Initializing Advanced Game Engine v1.12 Enhanced Edition\n");
    init_advanced_engine();
    return g_initialized ? 1 : 0;
}

EMSCRIPTEN_KEEPALIVE
void wasm_update_advanced_frame(double current_time) {
    if (!g_initialized || !g_game_state) return;
    update_advanced_game_logic(current_time);
}

EMSCRIPTEN_KEEPALIVE
void wasm_handle_advanced_key(int key_code, int pressed) {
    if (!g_initialized) return;
    handle_advanced_input(key_code, pressed == 1);
}

EMSCRIPTEN_KEEPALIVE
int wasm_get_advanced_score() {
    return g_game_state ? g_game_state->score : 0;
}

EMSCRIPTEN_KEEPALIVE
int wasm_get_advanced_entity_count() {
    return g_game_state ? g_game_state->entity_count : 0;
}

EMSCRIPTEN_KEEPALIVE
int wasm_get_particle_count() {
    return g_game_state ? g_game_state->particle_count : 0;
}

EMSCRIPTEN_KEEPALIVE
float wasm_get_frame_time() {
    return g_game_state ? g_game_state->performance.frame_time_ms : 0.0f;
}

EMSCRIPTEN_KEEPALIVE
void wasm_set_graphics_quality(int quality) {
    if (!g_game_state) return;
    
    switch (quality) {
        case 0: // Low
            g_game_state->graphics.shadow_quality = 0;
            g_game_state->graphics.bloom_enabled = false;
            g_game_state->graphics.ssao_enabled = false;
            g_game_state->graphics.pbr_enabled = false;
            break;
        case 1: // Medium
            g_game_state->graphics.shadow_quality = 1;
            g_game_state->graphics.bloom_enabled = true;
            g_game_state->graphics.ssao_enabled = false;
            g_game_state->graphics.pbr_enabled = true;
            break;
        case 2: // High
            g_game_state->graphics.shadow_quality = 2;
            g_game_state->graphics.bloom_enabled = true;
            g_game_state->graphics.ssao_enabled = true;
            g_game_state->graphics.pbr_enabled = true;
            g_game_state->graphics.reflections_enabled = true;
            break;
    }
    printf("Graphics quality set to %d\n", quality);
}

EMSCRIPTEN_KEEPALIVE
void wasm_enable_multiplayer(const char* server_url) {
    if (!g_game_state) return;
    
    strncpy(g_game_state->network.server_url, server_url, sizeof(g_game_state->network.server_url) - 1);
    g_game_state->network.multiplayer_enabled = true;
    printf("Multiplayer enabled, connecting to: %s\n", server_url);
}

EMSCRIPTEN_KEEPALIVE
void wasm_cleanup_advanced() {
    printf("Advanced Game Engine v1.12 cleaned up\n");
    deallocate_game_state();
    g_initialized = false;
}
