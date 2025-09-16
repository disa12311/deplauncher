#include <emscripten.h>
#include <emscripten/html5.h>
#include <emscripten/threading.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>
#include <stdbool.h>

// Game Engine for Deplauncher 1.8 - Classic Edition
// Cleaned and optimized for better maintainability and performance

// === CONSTANTS ===
#define MAX_ENTITIES 1000
#define CANVAS_WIDTH 800.0f
#define CANVAS_HEIGHT 600.0f
#define ENTITY_NAME_SIZE 32
#define COLLISION_RADIUS 32.0f
#define MOVE_SPEED 200.0f
#define ROTATION_SPEED 45.0f
#define FRICTION 0.95f

// === CORE STRUCTURES ===

// 2D Vector for classic 2D gameplay
typedef struct {
    float x, y;
} Vector2;

// Entity structure for classic gameplay
typedef struct {
    Vector2 position;
    Vector2 velocity;
    float rotation;
    int texture_id;
    bool active;
    int health;
    int max_health;
    char name[ENTITY_NAME_SIZE];
    char tag[ENTITY_NAME_SIZE];
} Entity;

// Camera system
typedef struct {
    Vector2 position;
    float zoom;
    Vector2 target;
    float follow_speed;
} Camera;

// Performance metrics
typedef struct {
    double last_frame_time;
    int fps_counter;
    double fps_timer;
    float average_fps;
} PerformanceMetrics;

// Game state structure
typedef struct {
    Entity entities[MAX_ENTITIES];
    int entity_count;
    Camera camera;
    PerformanceMetrics performance;
    int score;
    int level;
    bool paused;
    bool debug_mode;
} GameState;

// === GLOBAL STATE ===
static GameState* g_game_state = NULL;
static bool g_initialized = false;

// === UTILITY FUNCTIONS ===

// Vector2 operations
Vector2 vec2_create(float x, float y) {
    Vector2 v = {x, y};
    return v;
}

Vector2 vec2_add(Vector2 a, Vector2 b) {
    return vec2_create(a.x + b.x, a.y + b.y);
}

Vector2 vec2_subtract(Vector2 a, Vector2 b) {
    return vec2_create(a.x - b.x, a.y - b.y);
}

Vector2 vec2_multiply_scalar(Vector2 v, float scalar) {
    return vec2_create(v.x * scalar, v.y * scalar);
}

float vec2_magnitude(Vector2 v) {
    return sqrtf(v.x * v.x + v.y * v.y);
}

Vector2 vec2_normalize(Vector2 v) {
    float mag = vec2_magnitude(v);
    if (mag > 0.001f) {
        return vec2_multiply_scalar(v, 1.0f / mag);
    }
    return vec2_create(0, 0);
}

float vec2_distance(Vector2 a, Vector2 b) {
    return vec2_magnitude(vec2_subtract(a, b));
}

// Math utilities
float lerp(float a, float b, float t) {
    return a + t * (b - a);
}

float clamp(float value, float min_val, float max_val) {
    if (value < min_val) return min_val;
    if (value > max_val) return max_val;
    return value;
}

// === MEMORY MANAGEMENT ===

bool allocate_game_state() {
    g_game_state = (GameState*)calloc(1, sizeof(GameState));
    return g_game_state != NULL;
}

void deallocate_game_state() {
    if (g_game_state) {
        free(g_game_state);
        g_game_state = NULL;
    }
}

// === ENTITY MANAGEMENT ===

Entity* create_entity(Vector2 position, int texture_id, const char* name) {
    if (!g_game_state || g_game_state->entity_count >= MAX_ENTITIES) {
        return NULL;
    }
    
    Entity* entity = &g_game_state->entities[g_game_state->entity_count++];
    
    // Initialize entity
    entity->position = position;
    entity->velocity = vec2_create(0, 0);
    entity->rotation = 0.0f;
    entity->texture_id = texture_id;
    entity->active = true;
    entity->health = 100;
    entity->max_health = 100;
    strncpy(entity->name, name, ENTITY_NAME_SIZE - 1);
    entity->name[ENTITY_NAME_SIZE - 1] = '\0';
    strcpy(entity->tag, "Default");
    
    return entity;
}

Entity* find_entity_by_name(const char* name) {
    if (!g_game_state) return NULL;
    
    for (int i = 0; i < g_game_state->entity_count; i++) {
        if (g_game_state->entities[i].active && 
            strcmp(g_game_state->entities[i].name, name) == 0) {
            return &g_game_state->entities[i];
        }
    }
    return NULL;
}

Entity* find_entity_by_tag(const char* tag) {
    if (!g_game_state) return NULL;
    
    for (int i = 0; i < g_game_state->entity_count; i++) {
        if (g_game_state->entities[i].active && 
            strcmp(g_game_state->entities[i].tag, tag) == 0) {
            return &g_game_state->entities[i];
        }
    }
    return NULL;
}

void destroy_entity(Entity* entity) {
    if (entity) {
        entity->active = false;
    }
}

// === ENTITY SYSTEMS ===

void update_entity_physics(Entity* entity, float delta_time) {
    if (!entity || !entity->active) return;
    
    // Apply velocity
    entity->position = vec2_add(entity->position, vec2_multiply_scalar(entity->velocity, delta_time));
    
    // Apply rotation
    entity->rotation += delta_time * ROTATION_SPEED;
    if (entity->rotation > 360.0f) entity->rotation -= 360.0f;
    
    // Boundary wrapping for classic arcade feel
    if (entity->position.x < 0) entity->position.x = CANVAS_WIDTH;
    if (entity->position.x > CANVAS_WIDTH) entity->position.x = 0;
    if (entity->position.y < 0) entity->position.y = CANVAS_HEIGHT;
    if (entity->position.y > CANVAS_HEIGHT) entity->position.y = 0;
    
    // Apply friction
    entity->velocity = vec2_multiply_scalar(entity->velocity, FRICTION);
}

void update_entity_ai(Entity* entity, float delta_time, double current_time) {
    if (!entity || !entity->active) return;
    if (strcmp(entity->tag, "Player") == 0) return; // Skip player
    
    // Simple AI: circular movement pattern
    float time_factor = current_time * 0.001f;
    float move_speed = 50.0f;
    
    Vector2 ai_velocity = vec2_create(
        sinf(time_factor + entity->texture_id) * move_speed,
        cosf(time_factor + entity->texture_id) * move_speed
    );
    
    entity->velocity = ai_velocity;
}

// === COLLISION SYSTEM ===

bool check_circle_collision(Vector2 pos_a, Vector2 pos_b, float radius) {
    return vec2_distance(pos_a, pos_b) < radius;
}

void resolve_collision(Entity* entity_a, Entity* entity_b) {
    if (!entity_a || !entity_b || !entity_a->active || !entity_b->active) return;
    
    Vector2 direction = vec2_subtract(entity_a->position, entity_b->position);
    float distance = vec2_magnitude(direction);
    
    if (distance < COLLISION_RADIUS) {
        // Separate entities
        float overlap = COLLISION_RADIUS - distance;
        Vector2 separation = vec2_multiply_scalar(vec2_normalize(direction), overlap * 0.5f);
        
        entity_a->position = vec2_add(entity_a->position, separation);
        entity_b->position = vec2_subtract(entity_b->position, separation);
        
        // Apply bounce effect
        float bounce_force = 100.0f;
        Vector2 bounce_direction = vec2_normalize(direction);
        
        entity_a->velocity = vec2_add(entity_a->velocity, 
            vec2_multiply_scalar(bounce_direction, bounce_force));
        entity_b->velocity = vec2_subtract(entity_b->velocity, 
            vec2_multiply_scalar(bounce_direction, bounce_force));
        
        // Handle collision logic
        if (strcmp(entity_a->tag, "Player") == 0) {
            g_game_state->score += 10;
        } else if (strcmp(entity_b->tag, "Player") == 0) {
            g_game_state->score += 10;
        }
    }
}

void update_collision_system() {
    if (!g_game_state) return;
    
    for (int i = 0; i < g_game_state->entity_count; i++) {
        for (int j = i + 1; j < g_game_state->entity_count; j++) {
            resolve_collision(&g_game_state->entities[i], &g_game_state->entities[j]);
        }
    }
}

// === CAMERA SYSTEM ===

void init_camera() {
    if (!g_game_state) return;
    
    g_game_state->camera.position = vec2_create(CANVAS_WIDTH / 2, CANVAS_HEIGHT / 2);
    g_game_state->camera.zoom = 1.0f;
    g_game_state->camera.target = vec2_create(CANVAS_WIDTH / 2, CANVAS_HEIGHT / 2);
    g_game_state->camera.follow_speed = 5.0f;
}

void update_camera(float delta_time) {
    if (!g_game_state) return;
    
    // Follow player if exists
    Entity* player = find_entity_by_tag("Player");
    if (player) {
        g_game_state->camera.target = player->position;
    }
    
    // Smooth camera movement
    float lerp_factor = 1.0f - expf(-g_game_state->camera.follow_speed * delta_time);
    g_game_state->camera.position.x = lerp(g_game_state->camera.position.x, 
        g_game_state->camera.target.x, lerp_factor);
    g_game_state->camera.position.y = lerp(g_game_state->camera.position.y, 
        g_game_state->camera.target.y, lerp_factor);
}

// === CLEANUP SYSTEM ===

void cleanup_inactive_entities() {
    if (!g_game_state) return;
    
    int write_index = 0;
    for (int read_index = 0; read_index < g_game_state->entity_count; read_index++) {
        if (g_game_state->entities[read_index].active) {
            if (write_index != read_index) {
                g_game_state->entities[write_index] = g_game_state->entities[read_index];
            }
            write_index++;
        }
    }
    g_game_state->entity_count = write_index;
}

// === PERFORMANCE SYSTEM ===

void update_performance_metrics(double current_time) {
    if (!g_game_state) return;
    
    PerformanceMetrics* perf = &g_game_state->performance;
    
    perf->fps_counter++;
    perf->fps_timer += (current_time - perf->last_frame_time) / 1000.0;
    
    if (perf->fps_timer >= 1.0) {
        perf->average_fps = (float)perf->fps_counter;
        
        if (g_game_state->debug_mode) {
            printf("FPS: %.1f, Entities: %d, Score: %d\n", 
                perf->average_fps, g_game_state->entity_count, g_game_state->score);
        }
        
        perf->fps_counter = 0;
        perf->fps_timer = 0.0;
    }
    
    perf->last_frame_time = current_time;
}

// === GAME INITIALIZATION ===

void create_initial_entities() {
    // Create player entity
    Entity* player = create_entity(
        vec2_create(CANVAS_WIDTH / 2, CANVAS_HEIGHT / 2), 0, "Player");
    if (player) {
        strcpy(player->tag, "Player");
        player->health = 100;
    }
    
    // Create environment entities
    for (int i = 0; i < 10; i++) {
        Vector2 pos = vec2_create(
            (float)(rand() % (int)CANVAS_WIDTH),
            (float)(rand() % (int)CANVAS_HEIGHT)
        );
        
        Entity* env = create_entity(pos, 1 + (rand() % 3), "Environment");
        if (env) {
            snprintf(env->name, ENTITY_NAME_SIZE, "Obj_%d", i);
            strcpy(env->tag, "Environment");
            env->health = 50 + (rand() % 50);
        }
    }
}

void init_game_engine() {
    if (!allocate_game_state()) {
        printf("Failed to allocate game state memory\n");
        return;
    }
    
    // Initialize game state
    g_game_state->entity_count = 0;
    g_game_state->score = 0;
    g_game_state->level = 1;
    g_game_state->paused = false;
    g_game_state->debug_mode = false;
    
    // Initialize camera
    init_camera();
    
    // Initialize performance metrics
    g_game_state->performance.last_frame_time = emscripten_get_now();
    g_game_state->performance.fps_counter = 0;
    g_game_state->performance.fps_timer = 0.0;
    g_game_state->performance.average_fps = 60.0f;
    
    // Create initial entities
    create_initial_entities();
    
    g_initialized = true;
    printf("Game Engine v1.8 Classic Edition initialized successfully\n");
    printf("Initial entity count: %d\n", g_game_state->entity_count);
}

// === MAIN UPDATE FUNCTION ===

void update_game_logic(double current_time) {
    if (!g_game_state || g_game_state->paused) return;
    
    float delta_time = (float)(current_time - g_game_state->performance.last_frame_time) / 1000.0f;
    
    // Cap delta time to prevent large jumps
    delta_time = clamp(delta_time, 0.0f, 0.033f); // Max 33ms per frame
    
    // Update all entity systems
    for (int i = 0; i < g_game_state->entity_count; i++) {
        Entity* entity = &g_game_state->entities[i];
        if (entity->active) {
            update_entity_physics(entity, delta_time);
            update_entity_ai(entity, delta_time, current_time);
        }
    }
    
    // Update collision system
    update_collision_system();
    
    // Update camera
    update_camera(delta_time);
    
    // Cleanup inactive entities
    cleanup_inactive_entities();
    
    // Update performance metrics
    update_performance_metrics(current_time);
}

// === INPUT HANDLING ===

void handle_input(int key_code, bool pressed) {
    if (!g_game_state) return;
    
    Entity* player = find_entity_by_tag("Player");
    if (!player) return;
    
    if (pressed) {
        switch (key_code) {
            case 87: // W key
            case 38: // Up arrow
                player->velocity.y = -MOVE_SPEED;
                break;
            case 83: // S key  
            case 40: // Down arrow
                player->velocity.y = MOVE_SPEED;
                break;
            case 65: // A key
            case 37: // Left arrow
                player->velocity.x = -MOVE_SPEED;
                break;
            case 68: // D key
            case 39: // Right arrow
                player->velocity.x = MOVE_SPEED;
                break;
            case 32: // Space bar
                g_game_state->paused = !g_game_state->paused;
                printf("Game %s\n", g_game_state->paused ? "paused" : "resumed");
                break;
            case 192: // Tilde key (~)
                g_game_state->debug_mode = !g_game_state->debug_mode;
                printf("Debug mode %s\n", g_game_state->debug_mode ? "enabled" : "disabled");
                break;
        }
    }
}

// === WASM EXPORTS ===

EMSCRIPTEN_KEEPALIVE
int wasm_init_game() {
    printf("Initializing Game Engine v1.8 Classic Edition\n");
    init_game_engine();
    return g_initialized ? 1 : 0;
}

EMSCRIPTEN_KEEPALIVE
void wasm_update_frame(double current_time) {
    if (!g_initialized || !g_game_state) return;
    update_game_logic(current_time);
}

EMSCRIPTEN_KEEPALIVE
void wasm_handle_key(int key_code, int pressed) {
    if (!g_initialized) return;
    handle_input(key_code, pressed == 1);
}

EMSCRIPTEN_KEEPALIVE
int wasm_get_score() {
    return g_game_state ? g_game_state->score : 0;
}

EMSCRIPTEN_KEEPALIVE
int wasm_get_entity_count() {
    return g_game_state ? g_game_state->entity_count : 0;
}

EMSCRIPTEN_KEEPALIVE
void wasm_pause_game(int paused) {
    if (g_game_state) {
        g_game_state->paused = paused == 1;
    }
}

EMSCRIPTEN_KEEPALIVE
float wasm_get_fps() {
    return g_game_state ? g_game_state->performance.average_fps : 0.0f;
}

EMSCRIPTEN_KEEPALIVE
void wasm_set_debug_mode(int enabled) {
    if (g_game_state) {
        g_game_state->debug_mode = enabled == 1;
    }
}

// Export entity data for JavaScript rendering
EMSCRIPTEN_KEEPALIVE
float* wasm_get_entity_positions() {
    if (!g_game_state) return NULL;
    
    static float positions[MAX_ENTITIES * 4]; // x, y, rotation, texture_id
    
    for (int i = 0; i < g_game_state->entity_count; i++) {
        Entity* entity = &g_game_state->entities[i];
        if (entity->active) {
            positions[i * 4] = entity->position.x;
            positions[i * 4 + 1] = entity->position.y;
            positions[i * 4 + 2] = entity->rotation;
            positions[i * 4 + 3] = (float)entity->texture_id;
        }
    }
    
    return positions;
}

EMSCRIPTEN_KEEPALIVE
float wasm_get_camera_x() {
    return g_game_state ? g_game_state->camera.position.x : 0;
}

EMSCRIPTEN_KEEPALIVE
float wasm_get_camera_y() {
    return g_game_state ? g_game_state->camera.position.y : 0;
}

EMSCRIPTEN_KEEPALIVE
void wasm_add_entity(float x, float y, int texture_id, const char* name) {
    if (g_game_state && strlen(name) > 0) {
        Entity* entity = create_entity(vec2_create(x, y), texture_id, name);
        if (entity) {
            printf("Added entity: %s at (%.1f, %.1f)\n", name, x, y);
        }
    }
}

EMSCRIPTEN_KEEPALIVE
void wasm_reset_game() {
    if (!g_game_state) return;
    
    printf("Resetting game state\n");
    g_game_state->entity_count = 0;
    g_game_state->score = 0;
    g_game_state->level = 1;
    g_game_state->paused = false;
    
    // Reinitialize entities
    create_initial_entities();
    
    // Reset camera
    init_camera();
}

EMSCRIPTEN_KEEPALIVE
void wasm_cleanup() {
    printf("Game Engine v1.8 cleaned up\n");
    deallocate_game_state();
    g_initialized = false;
}
