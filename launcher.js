// launcher.js - Game Launcher for Eaglercraft
// Tạo giao diện launcher với nút Play để mở file index.html

class GameLauncher {
    constructor() {
        this.gameWindow = null;
        this.isGameRunning = false;
        this.init();
    }

    init() {
        // Tạo CSS styles
        this.createStyles();
        
        // Tạo HTML structure
        this.createLauncherUI();
        
        // Bind events
        this.bindEvents();
        
        console.log('Game Launcher initialized successfully!');
    }

    createStyles() {
        const style = document.createElement('style');
        style.textContent = `
            /* Launcher Styles */
            .game-launcher {
                position: fixed;
                top: 0;
                left: 0;
                width: 100%;
                height: 100%;
                background: linear-gradient(135deg, #1a1a2e, #16213e, #0f3460);
                font-family: 'Arial', sans-serif;
                display: flex;
                justify-content: center;
                align-items: center;
                z-index: 10000;
            }

            .launcher-panel {
                background: rgba(255, 255, 255, 0.1);
                backdrop-filter: blur(15px);
                border-radius: 20px;
                padding: 40px;
                text-align: center;
                box-shadow: 0 20px 40px rgba(0, 0, 0, 0.3);
                border: 1px solid rgba(255, 255, 255, 0.2);
                max-width: 400px;
                width: 90%;
                color: white;
            }

            .game-title {
                font-size: 2.5em;
                font-weight: bold;
                margin-bottom: 10px;
                background: linear-gradient(45deg, #ff6b6b, #4ecdc4, #45b7d1);
                -webkit-background-clip: text;
                -webkit-text-fill-color: transparent;
                background-clip: text;
                text-shadow: 2px 2px 4px rgba(0, 0, 0, 0.3);
            }

            .game-subtitle {
                color: rgba(255, 255, 255, 0.8);
                margin-bottom: 30px;
                font-size: 1.1em;
            }

            .play-btn {
                background: linear-gradient(45deg, #ff6b6b, #ee5a52);
                border: none;
                color: white;
                padding: 15px 50px;
                font-size: 18px;
                font-weight: bold;
                border-radius: 50px;
                cursor: pointer;
                transition: all 0.3s ease;
                box-shadow: 0 4px 15px rgba(255, 107, 107, 0.4);
                text-transform: uppercase;
                letter-spacing: 1px;
                margin: 10px;
            }

            .play-btn:hover {
                transform: translateY(-3px);
                box-shadow: 0 8px 25px rgba(255, 107, 107, 0.6);
                background: linear-gradient(45deg, #ee5a52, #ff6b6b);
            }

            .play-btn:active {
                transform: translateY(-1px);
            }

            .play-btn:disabled {
                opacity: 0.6;
                cursor: not-allowed;
                transform: none;
            }

            .close-btn {
                background: linear-gradient(45deg, #666, #555);
                border: none;
                color: white;
                padding: 10px 30px;
                font-size: 16px;
                border-radius: 25px;
                cursor: pointer;
                transition: all 0.3s ease;
                margin: 10px;
            }

            .close-btn:hover {
                background: linear-gradient(45deg, #777, #666);
                transform: translateY(-2px);
            }

            .status-text {
                margin-top: 20px;
                font-size: 14px;
                color: rgba(255, 255, 255, 0.7);
                min-height: 20px;
            }

            .loading-spinner {
                display: inline-block;
                width: 16px;
                height: 16px;
                border: 2px solid rgba(255, 255, 255, 0.3);
                border-radius: 50%;
                border-top-color: #fff;
                animation: spin 1s linear infinite;
                margin-left: 8px;
            }

            @keyframes spin {
                to { transform: rotate(360deg); }
            }

            .info-panel {
                margin-top: 25px;
                padding: 15px;
                background: rgba(0, 0, 0, 0.2);
                border-radius: 10px;
                font-size: 13px;
                text-align: left;
                line-height: 1.5;
            }

            .hidden {
                display: none;
            }
        `;
        document.head.appendChild(style);
    }

    createLauncherUI() {
        const launcher = document.createElement('div');
        launcher.className = 'game-launcher';
        launcher.id = 'gameLauncher';
        
        launcher.innerHTML = `
            <div class="launcher-panel">
                <div class="game-title">EAGLERCRAFT</div>
                <div class="game-subtitle">Minecraft 1.12 WASM-GC</div>
                
                <button class="play-btn" id="playGameBtn">
                    ▶ PLAY GAME
                </button>
                
                <button class="close-btn" id="closeLauncherBtn">
                    ✕ ĐÓNG LAUNCHER
                </button>
                
                <div class="status-text" id="statusText"></div>
                
                <div class="info-panel">
                    <strong>🎮 Thông tin:</strong><br>
                    • File: index.html (WASM-GC)<br>
                    • Platform: Web Browser<br>
                    • Version: Minecraft 1.12<br>
                    • Status: Ready to Launch
                </div>
            </div>
        `;
        
        document.body.appendChild(launcher);
    }

    bindEvents() {
        const playBtn = document.getElementById('playGameBtn');
        const closeBtn = document.getElementById('closeLauncherBtn');
        
        playBtn.addEventListener('click', () => this.launchGame());
        closeBtn.addEventListener('click', () => this.closeLauncher());
        
        // Keyboard shortcuts
        document.addEventListener('keydown', (e) => {
            if (e.key === 'Enter' && !this.isGameRunning) {
                this.launchGame();
            }
            if (e.key === 'Escape') {
                this.closeLauncher();
            }
        });
    }

    setStatus(message, isLoading = false) {
        const statusText = document.getElementById('statusText');
        statusText.innerHTML = message + (isLoading ? '<span class="loading-spinner"></span>' : '');
    }

    launchGame() {
        const playBtn = document.getElementById('playGameBtn');
        
        if (this.isGameRunning) {
            this.setStatus('Game đang chạy...');
            return;
        }

        // Disable button
        playBtn.disabled = true;
        playBtn.innerHTML = '⏳ ĐANG KHỞI CHẠY...';
        this.setStatus('Đang mở game...', true);

        try {
            // Mở file index.html trong cửa sổ mới
            this.gameWindow = window.open(
                './index.html', // Đường dẫn tới file index.html
                'EaglercraftGame',
                'width=1200,height=800,scrollbars=no,resizable=yes,status=no,toolbar=no,menubar=no,location=no'
            );

            if (this.gameWindow) {
                this.isGameRunning = true;
                this.setStatus('✅ Game đã được mở trong cửa sổ mới!');
                
                // Monitor window close
                const checkClosed = setInterval(() => {
                    if (this.gameWindow.closed) {
                        this.isGameRunning = false;
                        playBtn.disabled = false;
                        playBtn.innerHTML = '▶ PLAY GAME';
                        this.setStatus('Game đã đóng.');
                        clearInterval(checkClosed);
                    }
                }, 1000);
                
            } else {
                throw new Error('Không thể mở cửa sổ game. Popup có thể bị chặn.');
            }
            
        } catch (error) {
            console.error('Lỗi khi mở game:', error);
            this.setStatus('❌ Lỗi: ' + error.message);
            playBtn.disabled = false;
            playBtn.innerHTML = '▶ PLAY GAME';
        }
        
        // Reset button sau 3 giây nếu không có lỗi
        setTimeout(() => {
            if (this.isGameRunning) {
                playBtn.disabled = false;
                playBtn.innerHTML = '🎮 GAME RUNNING';
            }
        }, 3000);
    }

    closeLauncher() {
        const launcher = document.getElementById('gameLauncher');
        
        if (this.isGameRunning && this.gameWindow && !this.gameWindow.closed) {
            const confirmClose = confirm('Game đang chạy. Bạn có muốn đóng cả game và launcher?');
            if (confirmClose) {
                this.gameWindow.close();
                this.isGameRunning = false;
            } else {
                return;
            }
        }
        
        launcher.style.animation = 'fadeOut 0.3s ease-out';
        setTimeout(() => {
            launcher.remove();
        }, 300);
    }
}

// Auto-initialize khi DOM loaded
document.addEventListener('DOMContentLoaded', function() {
    new GameLauncher();
});

// Hoặc khởi tạo ngay lập tức nếu DOM đã sẵn sàng
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', function() {
        new GameLauncher();
    });
} else {
    new GameLauncher();
}

// Export cho sử dụng global
window.GameLauncher = GameLauncher;

// CSS animation
const fadeOutStyle = document.createElement('style');
fadeOutStyle.textContent = `
    @keyframes fadeOut {
        from { opacity: 1; transform: scale(1); }
        to { opacity: 0; transform: scale(0.9); }
    }
`;
document.head.appendChild(fadeOutStyle);
