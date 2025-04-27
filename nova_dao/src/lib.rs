use anchor_lang::prelude::*; 
use anchor_lang::solana_program::system_program;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod nova_dao {
    use super::*;

    // Initialize DAO
    pub fn initialize_dao(ctx: Context<InitializeDao>, name: String) -> Result<()> {
        let dao = &mut ctx.accounts.dao;
        dao.authority = ctx.accounts.authority.key();
        dao.name = name;
        dao.member_count = 1;
        Ok(())
    }

    // Add member to DAO
    pub fn add_member(ctx: Context<AddMember>, username: String) -> Result<()> {
        let member = &mut ctx.accounts.member;
        let dao = &mut ctx.accounts.dao;

        member.dao = dao.key();
        member.authority = ctx.accounts.user.key();
        member.username = username;
        member.tasks_completed = 0;

        dao.member_count += 1;
        Ok(())
    }

    // Create task
    pub fn create_task(ctx: Context<CreateTask>, title: String, description: String) -> Result<()> {
        let task = &mut ctx.accounts.task;
        let dao = &ctx.accounts.dao;

        task.dao = dao.key();
        task.creator = ctx.accounts.authority.key();
        task.title = title;
        task.description = description;
        task.status = TaskStatus::Open;
        task.assignee = None;
        Ok(())
    }

    // Assign task
    pub fn assign_task(ctx: Context<AssignTask>) -> Result<()> {
        let task = &mut ctx.accounts.task;
        let member = &ctx.accounts.member;

        require!(task.status == TaskStatus::Open, CustomError::TaskNotAvailable);
        
        task.status = TaskStatus::InProgress;
        task.assignee = Some(member.key());
        Ok(())
    }

    // Submit task completion
    pub fn submit_task(ctx: Context<SubmitTask>) -> Result<()> {
        let task = &mut ctx.accounts.task;
        let member = &mut ctx.accounts.member;

        require!(task.assignee == Some(member.key()), CustomError::NotAssigned);
        
        task.status = TaskStatus::PendingReview;
        Ok(())
    }

    // Approve or reject task completion
    pub fn review_task(ctx: Context<ReviewTask>, approved: bool) -> Result<()> {
        let task = &mut ctx.accounts.task;
        let member = &mut ctx.accounts.member;

        if approved {
            task.status = TaskStatus::Completed;
            member.tasks_completed += 1;
        } else {
            task.status = TaskStatus::Open;
            task.assignee = None;
        }
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeDao<'info> {
    #[account(
        init,
        payer = authority,
        space = DaoState::LEN
    )]
    pub dao: Account<'info, DaoState>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AddMember<'info> {
    #[account(mut)]
    pub dao: Account<'info, DaoState>,
    #[account(
        init,
        payer = authority,
        space = MemberState::LEN
    )]
    pub member: Account<'info, MemberState>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateTask<'info> {
    #[account(mut)]
    pub dao: Account<'info, DaoState>,
    #[account(
        init,
        payer = authority,
        space = TaskState::LEN
    )]
    pub task: Account<'info, TaskState>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AssignTask<'info> {
    #[account(mut)]
    pub task: Account<'info, TaskState>,
    pub member: Account<'info, MemberState>,
    #[account(mut)]
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct SubmitTask<'info> {
    #[account(mut)]
    pub task: Account<'info, TaskState>,
    #[account(mut)]
    pub member: Account<'info, MemberState>,
    #[account(mut)]
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct ReviewTask<'info> {
    #[account(mut)]
    pub task: Account<'info, TaskState>,
    #[account(mut)]
    pub member: Account<'info, MemberState>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

#[account]
pub struct DaoState {
    pub authority: Pubkey,
    pub name: String,
    pub member_count: u64,
}

#[account]
pub struct MemberState {
    pub dao: Pubkey,
    pub authority: Pubkey,
    pub username: String,
    pub tasks_completed: u64,
}

#[account]
pub struct TaskState {
    pub dao: Pubkey,
    pub creator: Pubkey,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub assignee: Option<Pubkey>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum TaskStatus {
    Open,
    InProgress,
    PendingReview,
    Completed,
}

#[error_code]
pub enum CustomError {
    TaskNotAvailable,
    NotAssigned,
}

impl DaoState {
    pub const LEN: usize = 8 + 32 + 200 + 8;
}

impl MemberState {
    pub const LEN: usize = 8 + 32 + 32 + 100 + 8;
}

impl TaskState {
    pub const LEN: usize = 8 + 32 + 32 + 100 + 500 + 1 + 33;
}