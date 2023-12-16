## Mathematical model
### Approach food
To model the approaching manner of slime mold in the mathematical model of SMA, as a mathematical equation, the next rule is developed to make the start to contraction mode:
$$
X_{t+1}=\left\{\begin{matrix}
X_{b}(t)+v_{b}.\left ( W.X_{A}(t)-X_{B}(t) \right )
 & r<p \\ 
 v_{c}.X_{t} & r\geq p
\end{matrix}\right.
\tag{1}
$$
where $v_{b}$ is a parameter with a interval of $[-a,a]$, $v_{c}$ decreases linearly from one to zero. $t$ denotes the current iteration, $X_{b}$ shows the individual position with the highest odor concentration currently explored, $X$ is the location vector of slime mold, $X_{A}$ and $X_{B}$ are two individuals, that we randomly selected from the current population, $W$ is the weight of slime mold. The equation of $p$ is as follows:
$$
p= \tanh \left |S(i)-DF|  \right.
\tag{2}
$$
where $\in{1,2,\cdots,n}$, $S(i)$ is the fitness of $X$ , ${DF}$ is the best fitness attained in all iterations.
The formula of $v_{b}$ can be expressed as follows: 
$$
v_{b}=[-a,a]
\tag{3}
$$
$$
a=  \arctan h \left ( -\left ( \frac{t}{max_t} \right )+1 \right )
\tag{4}
$$
The formula of $W$ can be expressed as follows:
$$
W(SmellIndex(i))=\left\{\begin{matrix}
1+r \log ((b_F-S(i))/(b_F-w_F)+1) & condition  \\ 
1-r \log ((b_F-S(i))/(b_F-w_F)+1) & others
\end{matrix} \right.
\tag{5}
$$
$$
SmellIndex=sort(S)
\tag{6}
$$
where $condition$ show that $S(i)$ ranks first half of the swarm, $r$ is the random value in the limit of $[0,1]$, $b_F$ is the best fitness attained in the current loop, $w_F$ is the worst fitness value attained in the iterative procedure, $SmellIndex$ is the sequence of fitness values sorted (ascends in the minimum value case).
### Wrap food
The mathematical rule for the update on the location of slime mold is as follows: 
$$
X^{*}=\left\{\begin{matrix}
rand (UB-LB)+LB & rand<z   \\ 
 X_b (t)+v_b(WX_A(t)-X_B (t)) & r<p\\ 
v_c X(t) & r\geq p
\end{matrix}\right.
\tag{7}
$$
where $LB$ and $UB$ denote the lower and upper limits of the feature range, rand and $r$ is the random value in $[0,1]$.
### Oscillation
The value of $v_b$ oscillates in a random manner between $[-a,a]$ and gradually approaches zero with more iterations. The value of $v_c$ oscillates among $[-1,1]$ and converges to zero eventually.
## The SMA algorithm
* **Inputs**: The population size $N$ and maximum number of iterations $max_t$ 
* **Outputs**: The best solution Initialize the the positions of slime mold $X_{i}(i=1,2,\ldots,n)$ 
	* Calculate the fitness of all slime mold 
	* Calculate the $W$ by Eq. (5) 
	* Update $p$, $v_b$, $v_c$; 
	* Update positions by Eq. (7) 
* **Returns**: bestFitness and $X_{b}$

```matlab
N = 30; % Number of search agents

Function_name = 'F1'; % Name of the test function, range from F1-F13

T = 500; % Maximum number of iterations

dimSize = 30; %dimension size

% Load details of the selected benchmark function
% [-100, 100, dimSize, F1]
[lb, ub, dim, fobj] = Get_Functions_SMA(Function_name, dimSize);

[Destination_fitness, bestPositions, Convergence_curve] = SMA(N, T, lb, ub, dim, fobj);

% This function initialize the first population of search agents
function Positions = initialization(SearchAgents_no, dim, ub, lb)

    Boundary_no = size(ub, 2); % numnber of boundaries

    % If the boundaries of all variables are equal and user enter a signle
    % number for both ub and lb
    if Boundary_no == 1
        Positions = rand(SearchAgents_no, dim) .* (ub - lb) + lb;
    end

    % If each variable has a different lb and ub
    if Boundary_no > 1

        for i = 1:dim
            ub_i = ub(i);
            lb_i = lb(i);
            Positions(:, i) = rand(SearchAgents_no, 1) .* (ub_i - lb_i) + lb_i;
        end

    end

end

% Max_iter: maximum iterations, N: populatoin size, Convergence_curve: Convergence curve
% To run SMA: [Destination_fitness,bestPositions,Convergence_curve]=SMA(N,Max_iter,lb,ub,dim,fobj)
function [Destination_fitness, bestPositions, Convergence_curve] = SMA(N, Max_iter, lb, ub, dim, fobj)
    disp('SMA is now tackling your problem')

    % initialize position
    bestPositions = zeros(1, dim);
    Destination_fitness = inf; %change this to -inf for maximization problems
    AllFitness = inf * ones(N, 1); %record the fitness of all slime mold
    weight = ones(N, dim); %fitness weight of each slime mold
    %Initialize the set of random solutions
    X = initialization(N, dim, ub, lb);
    Convergence_curve = zeros(1, Max_iter);
    it = 1; %Number of iterations
    lb = ones(1, dim) .* lb; % lower boundary
    ub = ones(1, dim) .* ub; % upper boundary
    z = 0.03; % parameter

    % Main loop
    while it <= Max_iter

        %sort the fitness
        for i = 1:N
            % Check if solutions go outside the search space and bring them back
            Flag4ub = X(i, :) > ub;
            Flag4lb = X(i, :) < lb;
            X(i, :) = (X(i, :) .* (~(Flag4ub + Flag4lb))) + ub .* Flag4ub + lb .* Flag4lb;
            AllFitness(i) = fobj(X(i, :));
        end

        [SmellOrder, SmellIndex] = sort(AllFitness); %Eq.(2.6)
        worstFitness = SmellOrder(N);
        bestFitness = SmellOrder(1);

        S = bestFitness - worstFitness + eps; % plus eps to avoid denominator zero

        %calculate the fitness weight of each slime mold
        for i = 1:N

            for j = 1:dim

                if i <= (N / 2) %Eq.(2.5)
                    weight(SmellIndex(i), j) =
	                     1 + rand() * log10((bestFitness - SmellOrder(i)) / (S) + 1);
                else
                    weight(SmellIndex(i), j) =
	                     1 - rand() * log10((bestFitness - SmellOrder(i)) / (S) + 1);
                end

            end

        end

        %update the best fitness value and best position
        if bestFitness < Destination_fitness
            bestPositions = X(SmellIndex(1), :);
            Destination_fitness = bestFitness;
        end

        a = atanh(- (it / Max_iter) + 1); %Eq.(2.4)
        b = 1 - it / Max_iter;
        % Update the Position of search agents
        for i = 1:N

            if rand < z %Eq.(2.7)
                X(i, :) = (ub - lb) * rand + lb;
            else
                p = tanh(abs(AllFitness(i) - Destination_fitness)); %Eq.(2.2)
                vb = unifrnd(-a, a, 1, dim); %Eq.(2.3)
                vc = unifrnd(-b, b, 1, dim);

                for j = 1:dim
                    r = rand();
                    A = randi([1, N]); % two positions randomly selected from population
                    B = randi([1, N]);

                    if r < p %Eq.(2.1)
                        X(i, j) = bestPositions(j) + vb(j) * (weight(i, j) * X(A, j) - X(B, j));
                    else
                        X(i, j) = vc(j) * X(i, j);
                    end

                end

            end

        end

        Convergence_curve(it) = Destination_fitness;
        it = it + 1;
    end

end


function [lb, ub, dim, fobj] = Get_Functions_SMA(F, DimValue)
    switch F
        case 'F1'
            fobj = @F1;
            lb = -100;
            ub = 100;
            dim = DimValue;
    end
end

% F1
function o = F1(x)
    o = sum(x .^ 2);
end
```
### `Get_Function_SMA`
```matlab
function [lb, ub, dim, fobj] = Get_Functions_SMA(F, DimValue)

    switch F
        case 'F1'
            fobj = @F1;
            lb = -100;
            ub = 100;
            dim = DimValue;

        case 'F2'
            fobj = @F2;
            lb = -10;
            ub = 10;
            dim = DimValue;

        case 'F3'
            fobj = @F3;
            lb = -100;
            ub = 100;
            dim = DimValue;

        case 'F4'
            fobj = @F4;
            lb = -100;
            ub = 100;
            dim = DimValue;

        case 'F5'
            fobj = @F5;
            lb = -30;
            ub = 30;
            dim = DimValue;

        case 'F6'
            fobj = @F6;
            lb = -100;
            ub = 100;
            dim = DimValue;

        case 'F7'
            fobj = @F7;
            lb = -1.28;
            ub = 1.28;
            dim = DimValue;

        case 'F8'
            fobj = @F8;
            lb = -500;
            ub = 500;
            dim = DimValue;

        case 'F9'
            fobj = @F9;
            lb = -5.12;
            ub = 5.12;
            dim = DimValue;

        case 'F10'
            fobj = @F10;
            lb = -32;
            ub = 32;
            dim = DimValue;

        case 'F11'
            fobj = @F11;
            lb = -600;
            ub = 600;
            dim = DimValue;

        case 'F12'
            fobj = @F12;
            lb = -50;
            ub = 50;
            dim = DimValue;

        case 'F13'
            fobj = @F13;
            lb = -50;
            ub = 50;
            dim = DimValue;
    end

end

% F1

function o = F1(x)
    o = sum(x .^ 2);
end

% F2

function o = F2(x)
    o = sum(abs(x)) + prod(abs(x));
end

% F3

function o = F3(x)
    dim = size(x, 2);
    o = 0;

    for i = 1:dim
        o = o + sum(x(1:i)) ^ 2;
    end

end

% F4

function o = F4(x)
    o = max(abs(x));
end

% F5

function o = F5(x)
    dim = size(x, 2);
    o = sum(100 * (x(2:dim) - (x(1:dim - 1) .^ 2)) .^ 2 + (x(1:dim - 1) - 1) .^ 2);
end

% F6

function o = F6(x)
    o = sum(abs((x + .5)) .^ 2);
end

% F7

function o = F7(x)
    dim = size(x, 2);
    o = sum([1:dim] .* (x .^ 4)) + rand;
end

% F8

function o = F8(x)
    o = sum(-x .* sin(sqrt(abs(x))));
end

% F9

function o = F9(x)
    dim = size(x, 2);
    o = sum(x .^ 2 - 10 * cos(2 * pi .* x)) + 10 * dim;
end

% F10

function o = F10(x)
    dim = size(x, 2);
    o = -20 * exp(- .2 * sqrt(sum(x .^ 2) / dim)) - exp(sum(cos(2 * pi .* x)) / dim) + 20 + exp(1);
end

% F11

function o = F11(x)
    dim = size(x, 2);
    o = sum(x .^ 2) / 4000 - prod(cos(x ./ sqrt([1:dim]))) + 1;
end

% F12

function o = F12(x)
    dim = size(x, 2);
    o = (pi / dim) * (10 * ((sin(pi * (1 + (x(1) + 1) / 4))) ^ 2) + sum((((x(1:dim - 1) + 1) ./ 4) .^ 2) .* ...
        (1 + 10 .* ((sin(pi .* (1 + (x(2:dim) + 1) ./ 4)))) .^ 2)) + ((x(dim) + 1) / 4) ^ 2) + sum(Ufun(x, 10, 100, 4));
end

% F13

function o = F13(x)
    dim = size(x, 2);
    o = .1 * ((sin(3 * pi * x(1))) ^ 2 + sum((x(1:dim - 1) - 1) .^ 2 .* (1 + (sin(3 .* pi .* x(2:dim))) .^ 2)) + ...
        ((x(dim) - 1) ^ 2) * (1 + (sin(2 * pi * x(dim))) ^ 2)) + sum(Ufun(x, 5, 100, 4));
end

function o = Ufun(x, a, k, m)
    o = k .* ((x - a) .^ m) .* (x > a) + k .* ((-x - a) .^ m) .* (x < (-a));
end
```

